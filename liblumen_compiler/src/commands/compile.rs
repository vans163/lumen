use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc::channel};
use std::time::Instant;

use anyhow::anyhow;

use clap::ArgMatches;

use executors::Executor;
use executors::crossbeam_channel_pool::ThreadPool;

use log::debug;

use libeir_diagnostics::{CodeMap, Emitter};

use liblumen_codegen::{
    self as codegen,
    codegen::{CodegenResults, ProjectInfo},
};
use liblumen_codegen::linker::{self, LinkerInfo};
use liblumen_session::{CodegenOptions, DebuggingOptions, Options};
use liblumen_target::{self as target, Target};
use liblumen_util::time::HumanDuration;

use crate::compiler::{prelude::*, *};
use crate::commands::*;

pub fn handle_command<'a>(
    c_opts: CodegenOptions,
    z_opts: DebuggingOptions,
    matches: &ArgMatches<'a>,
    cwd: PathBuf,
    emitter: Option<Arc<dyn Emitter>>,
) -> anyhow::Result<()> {
    // Extract options from provided arguments
    let mut options = Options::new(c_opts, z_opts, cwd, &matches)?;
    // Construct empty code map for use in compilation
    let codemap = Arc::new(Mutex::new(CodeMap::new()));
    // Set up diagnostics
    let diagnostics = create_diagnostics_handler(&options, codemap.clone(), emitter);

    // Initialize codegen backend
    codegen::init(&options);

    let host = Target::search(target::host_triple()).unwrap_or_else(|e| {
        diagnostics
            .fatal_str(&format!(
                "Unable to load host specification: {}",
                e.to_string()
            ))
            .raise()
    });

    options.set_host_target(host);

    // Build query database
    let mut db = CompilerDatabase::new(codemap, diagnostics);

    // The core of the query system is the initial set of options provided to the compiler
    //
    // The query system will use these options to construct the set of inputs on demand
    db.set_options(Arc::new(options));

    let inputs = db.inputs().unwrap_or_else(abort_on_err);

    // Parse sources
    let num_inputs = inputs.len();
    if num_inputs < 1 {
        db.diagnostics().fatal_str("No input sources found!").raise();
    }

    let start = Instant::now();
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    for input in inputs.iter().cloned() {
        debug!("spawning worker for {:?}", input);
        let tx = tx.clone();
        let snapshot = db.snapshot();
        pool.execute(move || {
            let thread_id = std::thread::current().id();
            debug!("starting to compile on thread {:?}", thread_id);
            let compilation_result = snapshot.compile(input);
            debug!("compilation finished on thread {:?} {:?}", thread_id, &compilation_result);
            tx.send(compilation_result)
              .expect("worker failed: unable to send compiled module back to main thread");
        });
    }

    let options = db.options();
    let mut codegen_results = CodegenResults {
        project_name: options.project_name.clone(),
        modules: Vec::with_capacity(num_inputs),
        windows_subsystem: None,
        linker_info: LinkerInfo::new(),
        project_info: ProjectInfo::new(&options),
    };

    debug!("awaiting results from workers ({} units)", num_inputs);

    let diagnostics = db.diagnostics();
    let mut received = 0;
    loop {
        match rx.recv() {
            Ok(compile_result) => {
                debug!("received compilation result from worker: {:?}", &compile_result);
                if let Ok(compiled_module) = compile_result {
                    diagnostics.success("Compiled", &compiled_module.name());
                    codegen_results.modules.push(compiled_module);
                } else {
                    debug!("received compilation result from worker: compilation failed");
                }
                received += 1;
            }
            Err(_) => {
                debug!("received compilation result from worker: terminated");
                received += 1;
            }
        }
        if received == num_inputs {
            debug!("all compilation units are finished, terminating thread pool");
            if let Err(ref reason) = pool.shutdown() {
                diagnostics.fatal_str(reason).raise();
            }
            break;
        }
    }

    // Do not proceed to linking if there were compilation errors
    diagnostics.abort_if_errors();

    // Link all compiled objects
    if let Err(err) = linker::link_binary(&options, &diagnostics, &codegen_results) {
        diagnostics.error(err);
        return Err(anyhow!("failed to link binary"));
    }

    let duration = HumanDuration::since(start);
    diagnostics.success("Finished", &format!("built {} in {:#}", options.project_name, duration));
    Ok(())
}