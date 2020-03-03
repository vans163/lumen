mod with_arity;

use super::*;

#[test]
fn without_arity_when_run_exits_undef_and_sends_exit_message_to_parent() {
    let parent_arc_process = process::test_init();
    let arc_scheduler = Scheduler::current();

    let priority = Priority::Normal;
    let run_queue_length_before = arc_scheduler.run_queue_len(priority);

    let module = atom!("erlang");
    let function = atom!("+");

    // erlang.+/1 and erlang.+/2 exists so use 3 for invalid arity
    let arguments = parent_arc_process
        .list_from_slice(&[
            parent_arc_process.integer(0).unwrap(),
            parent_arc_process.integer(1).unwrap(),
            parent_arc_process.integer(2).unwrap(),
        ])
        .unwrap();

    let result = native(&parent_arc_process, module, function, arguments);

    assert!(result.is_ok());

    let returned = result.unwrap();
    let result_boxed_tuple: Result<Boxed<Tuple>, _> = returned.try_into();

    assert!(result_boxed_tuple.is_ok());

    let boxed_tuple: Boxed<Tuple> = result_boxed_tuple.unwrap();

    let child_pid_term = boxed_tuple[0];
    let child_result_pid: Result<Pid, _> = child_pid_term.try_into();

    assert!(child_result_pid.is_ok());

    let child_pid = child_result_pid.unwrap();

    let monitor_reference = boxed_tuple[1];

    assert!(monitor_reference.is_reference());

    let run_queue_length_after = arc_scheduler.run_queue_len(priority);

    assert_eq!(run_queue_length_after, run_queue_length_before + 1);

    let child_arc_process = pid_to_process(&child_pid).unwrap();

    assert!(arc_scheduler.run_through(&child_arc_process));
    assert!(!arc_scheduler.run_through(&child_arc_process));

    assert_eq!(child_arc_process.code_stack_len(), 1);
    assert_eq!(
        child_arc_process.current_module_function_arity(),
        Some(apply_3::module_function_arity())
    );
    assert_exits_undef(
        &child_arc_process,
        module,
        function,
        arguments,
        ":erlang.+/3 is not exported",
    );

    assert!(!parent_arc_process.is_exiting());

    let tag = atom!("DOWN");
    let reason = atom!("undef");

    assert_has_message!(
        &parent_arc_process,
        parent_arc_process
            .tuple_from_slice(&[
                tag,
                monitor_reference,
                Atom::str_to_term("process"),
                child_pid_term,
                reason
            ])
            .unwrap()
    );
}
