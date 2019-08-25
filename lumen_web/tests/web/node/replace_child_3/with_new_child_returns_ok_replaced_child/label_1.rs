use std::convert::TryInto;
use std::sync::Arc;

use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, ProcessControlBlock};
use liblumen_alloc::erts::term::{atom_unchecked, Boxed, Tuple};
use liblumen_alloc::ModuleFunctionArity;

use super::label_2;

pub fn place_frame(process: &ProcessControlBlock, placement: Placement) {
    process.place_frame(frame(), placement);
}

// Private

// ```elixir
// # label 1
// # pushed to stack: ()
// # returned form call: {:ok, document}
// # full stack: ({:ok, document})
// # returns: {:ok parent}
// {:ok, old_child} = Lumen.Web.Document.create_element(document, "table")
// {:ok, parent} = Lumen.Web.Document.create_element(parent_document, "div")
// :ok = Lumen.Web.Node.append_child(parent, old_child)
// {:ok, new_child} = Lumen.Web.Document.create_element(document, "ul");
// {:ok, replaced_child} = Lumen.Web.replace_child(parent, new_child, old_child)
// ```
fn code(arc_process: &Arc<ProcessControlBlock>) -> code::Result {
    arc_process.reduce();

    let ok_parent_document = arc_process.stack_pop().unwrap();
    assert!(
        ok_parent_document.is_tuple(),
        "ok_parent_document ({:?}) is not a tuple",
        ok_parent_document
    );
    let ok_parent_document_tuple: Boxed<Tuple> = ok_parent_document.try_into().unwrap();
    assert_eq!(ok_parent_document_tuple.len(), 2);
    assert_eq!(ok_parent_document_tuple[0], atom_unchecked("ok"));
    let parent_document = ok_parent_document_tuple[1];
    assert!(parent_document.is_resource_reference());

    label_2::place_frame_with_arguments(arc_process, Placement::Replace, parent_document)?;

    let old_child_tag = arc_process.binary_from_str("table")?;
    lumen_web::document::create_element_2::place_frame_with_arguments(
        arc_process,
        Placement::Push,
        parent_document,
        old_child_tag,
    )?;

    ProcessControlBlock::call_code(arc_process)
}

fn frame() -> Frame {
    let module_function_arity = Arc::new(ModuleFunctionArity {
        module: super::module(),
        function: super::function(),
        arity: 0,
    });

    Frame::new(module_function_arity, code)
}
