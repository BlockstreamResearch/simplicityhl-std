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

/// The minimum passed to the function under test in every case; only the sequence
/// value declared in the transaction varies between test cases.
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

// Raw BIP68 nSequence encodings. See rust-simplicity's `jets.c` (`parse_sequence`):
// bit 31 = disable flag (set => `None`), bit 22 = type flag (0 = blocks/Distance/Left,
// 1 = 512-second units/Duration/Right), low 16 bits = the declared value.
fn seq_disabled() -> Sequence {
    Sequence::MAX
}

fn seq_distance(blocks: u16) -> Sequence {
    Sequence::from_consensus(u32::from(blocks))
}

fn seq_duration(units: u16) -> Sequence {
    Sequence::from_consensus((1u32 << 22) | u32::from(units))
}

// ---------- shared: transaction version precondition ----------

#[simplex::test]
fn tx_version_below_2_fails(context: simplex::TestContext) -> anyhow::Result<()> {
    // `FinalTransaction` always builds a version 2 transaction and doesn't expose a
    // version setter, so the signer can't produce a version 1 transaction. Instead,
    // extract the PSET, override its version, and call `ProgramTrait::finalize`
    // directly: the same local Simplicity execution that `Signer::broadcast` runs
    // before touching the network.
    let program = program();
    let witness_values: WitnessValues = witness(EnforceRelativeDistance).into();

    let script = fund(&context, &program, DEFAULT_SEND_AMOUNT)?;
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(&script)?;

    // `extract_pst` only reads the inputs and outputs, so the program and witness
    // don't need to be attached here; they're passed to `finalize` below.
    let mut ft = FinalTransaction::new();
    ft.add_input(PartialInput::new(utxos[0].clone()), None);

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

// ---------- enforce_relative_distance ----------

#[simplex::test]
fn distance_disabled_sequence_is_pruned(context: simplex::TestContext) -> anyhow::Result<()> {
    // Disable flag set -> jet::parse_sequence returns None -> unwrap(None) hits the
    // pruned branch. Never reaches broadcast, so no mining needed.
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
    // Sequence declares a Duration (type flag set), but the function requires a
    // Distance -> unwrap_left panics on Right(_).
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
    // Declared distance is below MIN_DISTANCE -> le_16 assert fails.
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_distance(MIN_DISTANCE - 1),
        0,
        Expect::AssertFailed,
    )
}

#[simplex::test]
fn distance_equal_minimum_succeeds(context: simplex::TestContext) -> anyhow::Result<()> {
    // Mine enough blocks that the declared distance is satisfied on-chain, so the
    // node's own BIP68 check at broadcast also passes.
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
    // The declared distance satisfies MIN_DISTANCE, so local execution succeeds, but no
    // blocks are mined, so the real distance is short of the declared one and the node's
    // BIP68 check rejects the broadcast. This shows the guarantee is enforced by
    // consensus, not just by a value the spender writes into the transaction.
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDistance),
        seq_distance(MIN_DISTANCE),
        0,
        Expect::BroadcastRejected,
    )
}

// ---------- enforce_relative_duration ----------

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
    // Sequence declares a Distance, but the function requires a Duration ->
    // unwrap_right panics on Left(_).
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
    // This fails during local Simplicity execution, before the transaction reaches the
    // node, so real chain state can't affect the outcome. Unlike the distance version,
    // it can't also show that the spend fails when the real elapsed time does satisfy
    // the declared duration: that needs median-time-past to advance, which requires a
    // `setmocktime` helper that Simplex doesn't expose.
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
    // Duration counterpart of `distance_sufficient_declared_insufficient_real_is_rejected`.
    // Median-time-past not having advanced far enough is the default state right after
    // funding, so no mocktime is needed.
    run_with_sequence(
        &context,
        program(),
        witness(EnforceRelativeDuration),
        seq_duration(MIN_DURATION),
        0,
        Expect::BroadcastRejected,
    )
}

// There are no `duration_*_minimum_succeeds` tests: they need median-time-past to
// advance by MIN_DURATION * 512 seconds, which regtest can't do without a
// `setmocktime` helper that Simplex doesn't expose.
