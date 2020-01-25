mod with_atom_left;
mod with_big_integer_left;
mod with_empty_list_left;
mod with_external_pid_left;
mod with_float_left;
mod with_function_left;
mod with_heap_binary_left;
mod with_list_left;
mod with_local_pid_left;
mod with_local_reference_left;
mod with_map_left;
mod with_small_integer_left;
mod with_subbinary_left;
mod with_tuple_left;

use std::convert::TryInto;
use std::sync::Arc;

use proptest::arbitrary::any;
use proptest::prop_assert_eq;
use proptest::strategy::Just;
use proptest::test_runner::{Config, TestRunner};

use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::closure::Definition;
use liblumen_alloc::erts::term::prelude::*;
use liblumen_alloc::located_code;

use crate::otp::erlang::are_exactly_equal_2::native;
use crate::scheduler::with_process_arc;
use crate::test::strategy;
