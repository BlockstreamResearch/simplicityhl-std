use std::fmt::Debug;

use simplex::fuzz::proptest::prelude::{Just, any};
use simplex::fuzz::proptest::strategy::Strategy;
use simplex::fuzz::{Expect, Scenario};
use simplex::transaction::{FinalTransaction, PartialInput, PartialOutput, UTXO};
use simplex::{FuzzMode, TestContext};

use crate::common::fuzz::check_scenario;
use crate::utxo::op_return::case;
use crate::utxo::op_return::case::Case;

const DEFAULT_DATA: &[u8] = &[0, 1];

fn scenario(data: Option<&[u8]>) -> Scenario {
    let mut tx = FinalTransaction::new();
    tx.add_input(PartialInput::new(UTXO::default()), None);

    match data {
        Some(data) => tx.add_output(case::op_return_output(data)),
        None => tx.add_output(PartialOutput::default()),
    }

    Scenario::new(tx)
}

fn check<T: Debug + 'static>(
    context: &TestContext<FuzzMode>,
    op_return_data: Option<&[u8]>,
    output_indices: impl Strategy<Value = T> + 'static,
    build: impl Fn(T) -> Case + 'static,
    expect: Expect,
) {
    check_scenario(
        context,
        scenario(op_return_data),
        output_indices,
        build,
        expect,
    );
}

fn arb_non_zero_u32() -> impl Strategy<Value = u32> {
    any::<u32>().prop_filter("output index should not be zero", |index| *index != 0)
}

// 8. is_output_op_return
#[simplex::fuzz]
fn is_output_op_return_true(context: TestContext<FuzzMode>) {
    check(
        &context,
        Some(DEFAULT_DATA),
        Just(0_u32),
        |output_index| case::is_output_op_return(output_index, true),
        Expect::Accept,
    );
}

#[simplex::fuzz]
fn is_output_op_return_empty_index_false(context: TestContext<FuzzMode>) {
    check(
        &context,
        Some(DEFAULT_DATA),
        arb_non_zero_u32(),
        |output_index| case::is_output_op_return(output_index, false),
        Expect::Accept,
    );
}

#[simplex::fuzz]
fn is_output_op_return_false(context: TestContext<FuzzMode>) {
    check(
        &context,
        None,
        Just(0_u32),
        |output_index| case::is_output_op_return(output_index, false),
        Expect::Accept,
    );
}

// 10. assert_output_is_op_return
#[simplex::fuzz]
fn assert_output_is_op_return_pass(context: TestContext<FuzzMode>) {
    check(
        &context,
        Some(DEFAULT_DATA),
        Just(0_u32),
        case::assert_output_is_op_return,
        Expect::Accept,
    );
}

#[simplex::fuzz]
fn assert_output_is_op_return_empty_index_fail(context: TestContext<FuzzMode>) {
    check(
        &context,
        Some(DEFAULT_DATA),
        arb_non_zero_u32(),
        case::assert_output_is_op_return,
        Expect::Reject,
    );
}

#[simplex::fuzz]
fn assert_output_is_op_return_fail(context: TestContext<FuzzMode>) {
    check(
        &context,
        None,
        Just(0_u32),
        case::assert_output_is_op_return,
        Expect::Reject,
    );
}
