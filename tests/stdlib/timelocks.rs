use simplex::program::ProgramTrait;
use simplex::simplicityhl::WitnessValues;
use simplex::simplicityhl::elements::Sequence;
use simplex::transaction::{FinalTransaction, PartialInput};

use crate::common::core::{Expect, fund, run_with_sequence};
use crate::common::utxo_helper::DEFAULT_SEND_AMOUNT;

use simplicityhl_std::artifacts::tests::timelocks::TimelocksProgram as TestTimelocksProgram;
use simplicityhl_std::artifacts::tests::timelocks::derived_timelocks::{
    TimelocksArguments as TestTimelocksArguments, TimelocksWitness as TestTimelocksWitness,
};

use FunctionToTest::*;

/// Dispatch indices; these must match the `is_selected(N, ..)` arms in
/// `simf/tests/timelocks.simf`.
enum FunctionToTest {
    EnforceRelativeDistance,
    EnforceRelativeDuration,
}

const MIN_DISTANCE: u16 = 5;
const MIN_DURATION: u16 = 5;

fn program() -> TestTimelocksProgram {
    TestTimelocksProgram::new(&TestTimelocksArguments {})
}

fn witness(function: FunctionToTest) -> TestTimelocksWitness {
    TestTimelocksWitness {
        function_index: function as u8,
        min_distance: MIN_DISTANCE,
        min_duration: MIN_DURATION,
    }
}

fn seq_disabled() -> Sequence {
    Sequence::MAX
}

fn seq_distance(blocks: u16) -> Sequence {
    Sequence::from_consensus(u32::from(blocks))
}

/// Sets the BIP68 type flag (bit 22), so `units` counts 512-second intervals.
fn seq_duration(units: u16) -> Sequence {
    Sequence::from_consensus((1u32 << 22) | u32::from(units))
}

#[simplex::test]
fn tx_version_below_2_fails(context: simplex::TestContext) -> anyhow::Result<()> {
    // A version 1 transaction can declare a timelock that consensus won't enforce.
    // FinalTransaction always builds version 2, so set the version on the PSET
    // and finalize directly.
    let program = program();
    let witness_values: WitnessValues = witness(EnforceRelativeDistance).into();

    let script = fund(&context, &program, DEFAULT_SEND_AMOUNT)?;
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(&script)?;

    let mut ft = FinalTransaction::new();
    ft.add_input(
        PartialInput::new(utxos[0].clone()).with_sequence(seq_distance(MIN_DISTANCE)),
        None,
    );

    let (mut pst, _secrets) = ft.extract_pst()?;
    pst.global.tx_data.version = 1;

    let result = program
        .as_ref()
        .finalize(&pst, &witness_values, 0, context.get_network());

    let err = result.expect_err("expected tx version < 2 to fail Simplicity execution");
    assert!(
        err.to_string().contains("Jet failed during execution"),
        "unexpected error: {err}"
    );

    Ok(())
}

#[simplex::test]
fn distance_disabled_sequence_is_pruned(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_disabled(),
        0,
        Expect::PrunedBranch,
    )
}

#[simplex::test]
fn distance_wrong_variant_is_pruned(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_duration(0),
        0,
        Expect::PrunedBranch,
    )
}

#[simplex::test]
fn distance_below_minimum_fails(context: simplex::TestContext) -> anyhow::Result<()> {
    // Blocks are mined so that the real distance suffices: only the declared value counts.
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_distance(MIN_DISTANCE - 1),
        u64::from(MIN_DISTANCE),
        Expect::AssertFailed,
    )
}

#[simplex::test]
fn distance_equal_minimum_succeeds(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_distance(MIN_DISTANCE),
        u64::from(MIN_DISTANCE),
        Expect::Ok,
    )
}

#[simplex::test]
fn distance_above_minimum_succeeds(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_distance(MIN_DISTANCE + 1),
        u64::from(MIN_DISTANCE + 1),
        Expect::Ok,
    )
}

#[simplex::test]
fn distance_sufficient_declared_insufficient_real_is_rejected(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    // Local execution succeeds; the node's BIP68 check rejects the broadcast.
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_distance(MIN_DISTANCE),
        0,
        Expect::BroadcastRejected,
    )
}

#[simplex::test]
fn duration_disabled_sequence_is_pruned(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDuration),
        seq_disabled(),
        0,
        Expect::PrunedBranch,
    )
}

#[simplex::test]
fn duration_wrong_variant_is_pruned(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDuration),
        seq_distance(0),
        0,
        Expect::PrunedBranch,
    )
}

#[simplex::test]
fn duration_below_minimum_fails(context: simplex::TestContext) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDuration),
        seq_duration(MIN_DURATION - 1),
        0,
        Expect::AssertFailed,
    )
}

#[simplex::test]
fn duration_sufficient_declared_insufficient_real_is_rejected(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDuration),
        seq_duration(MIN_DURATION),
        0,
        Expect::BroadcastRejected,
    )
}

// Currently, the duration happy path can't be tested: it needs median-time-past to
// advance, and Simplex can't mine blocks with later timestamps.
