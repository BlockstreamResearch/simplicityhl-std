//! Fuzz cases run in [`reference_tx`], whose outputs the layout mutations shuffle and extend.
//!
//! The witness comes from the guided strategy of Simplex, which draws output and entry indices
//! from the size of the transaction. The expected result is computed from the mutated
//! transaction with [`output_null_datum`]: a required witness value for the functions that
//! return a value (Simplex also checks that the contract rejects a wrong one), and a spec for
//! the asserts.

use simplex::fuzz::{Expect, FuzzCase, FuzzProgramScenarioBuilder, Mutations, Scenario};
use simplex::{FuzzMode, TestContext};

use simplicityhl_std::artifacts::tests::utxo::op_return::OpReturnProgram as OpReturnTestProgram;
use simplicityhl_std::artifacts::tests::utxo::op_return::derived_op_return::OpReturnWitness as OpReturnTestWitness;

use crate::utxo::op_return::case::{
    FunctionToTest, OpReturnEntry, PROGRAM_NAME, output_null_datum, reference_tx,
};

const SPENT_INPUT: usize = 0;

/// Fuzzes `function` in the reference transaction with its outputs shuffled.
fn run(
    context: &TestContext<FuzzMode>,
    function: FunctionToTest,
    configure: impl FnOnce(FuzzProgramScenarioBuilder<'_, OpReturnTestProgram>),
    spec: Option<fn(&FuzzCase<'_>) -> Expect>,
) {
    let mut scenario = Scenario::new(reference_tx());

    configure(
        scenario
            .add::<OpReturnTestProgram>(PROGRAM_NAME)
            .spends(SPENT_INPUT)
            .bind_witness(move |witness, _| witness.function_index = function as u8),
    );

    let engine = context
        .engine(scenario)
        .mutations(Mutations::none().layout().everywhere());

    match spec {
        Some(spec) => engine.spec(spec),
        None => engine,
    }
    .run();
}

/// What `jet::output_null_datum` returns for the witness indices in the mutated transaction.
fn null_datum(witness: &OpReturnTestWitness, case: &FuzzCase<'_>) -> Option<Option<OpReturnEntry>> {
    output_null_datum(
        case.transaction(),
        witness.output_index,
        witness.entry_index,
    )
}

fn witness(case: &FuzzCase<'_>) -> OpReturnTestWitness {
    case.program::<OpReturnTestProgram>(PROGRAM_NAME)
        .witness()
        .clone()
}

fn accept_if(accept: bool) -> Expect {
    if accept {
        Expect::Accept
    } else {
        Expect::Reject
    }
}

// 0. get_entry_option
#[simplex::fuzz]
fn get_entry_option(context: TestContext<FuzzMode>) {
    run(
        &context,
        FunctionToTest::GetEntryOption,
        |program| {
            program.requires_witness(|witness, case| {
                witness.expected_entry = null_datum(witness, case).flatten();
            });
        },
        None,
    );
}

// 8. is_output_op_return
#[simplex::fuzz]
fn is_output_op_return(context: TestContext<FuzzMode>) {
    run(
        &context,
        FunctionToTest::IsOutputOpReturn,
        |program| {
            program.requires_witness(|witness, case| {
                witness.expected_bool = null_datum(witness, case).is_some();
            });
        },
        None,
    );
}

// 9. has_entry
#[simplex::fuzz]
fn has_entry(context: TestContext<FuzzMode>) {
    run(
        &context,
        FunctionToTest::HasEntry,
        |program| {
            program.requires_witness(|witness, case| {
                witness.expected_bool = null_datum(witness, case).flatten().is_some();
            });
        },
        None,
    );
}

// 10. assert_output_is_op_return
#[simplex::fuzz]
fn assert_output_is_op_return(context: TestContext<FuzzMode>) {
    run(
        &context,
        FunctionToTest::AssertOutputIsOpReturn,
        |_| {},
        Some(|case| accept_if(null_datum(&witness(case), case).is_some())),
    );
}

// 11. assert_has_entry
#[simplex::fuzz]
fn assert_has_entry(context: TestContext<FuzzMode>) {
    run(
        &context,
        FunctionToTest::AssertHasEntry,
        |_| {},
        Some(|case| accept_if(null_datum(&witness(case), case).flatten().is_some())),
    );
}
