use super::*;

mod with_exported_function;

#[test]
fn without_exported_function_when_run_exits_undef_and_parent_does_not_exit() {
    let parent_arc_process = process::test_init();

    let arc_scheduler = Scheduler::current();

    let priority = Priority::Normal;
    let run_queue_length_before = arc_scheduler.run_queue_len(priority);

    let module = atom!("erlang");
    // Rust name instead of Erlang name
    let function = atom!("number_or_badarith_1");

    let arguments = parent_arc_process
        .cons(parent_arc_process.integer(0).unwrap(), Term::NIL)
        .unwrap();

    let result = spawn_3::native(&parent_arc_process, module, function, arguments);

    assert!(result.is_ok());

    let child_pid = result.unwrap();
    let child_pid_result_pid: Result<Pid, _> = child_pid.try_into();

    assert!(child_pid_result_pid.is_ok());

    let child_pid_pid = child_pid_result_pid.unwrap();

    let run_queue_length_after = arc_scheduler.run_queue_len(priority);

    assert_eq!(run_queue_length_after, run_queue_length_before + 1);

    let child_arc_process = pid_to_process(&child_pid_pid).unwrap();

    assert!(arc_scheduler.run_through(&child_arc_process));
    assert!(!arc_scheduler.run_through(&child_arc_process));

    assert_eq!(child_arc_process.code_stack_len(), 1);

    let child_frame = child_arc_process.current_frame().unwrap();

    assert_eq!(child_frame.module, erlang::module());
    assert_eq!(
        child_frame.definition,
        Definition::Export {
            function: apply_3::function()
        }
    );
    assert_eq!(child_frame.arity, apply_3::ARITY);

    assert_exits_undef(
        &child_arc_process,
        module,
        function,
        arguments,
        ":erlang.number_or_badarith_1/1 is not exported",
    );

    assert!(!parent_arc_process.is_exiting());
}
