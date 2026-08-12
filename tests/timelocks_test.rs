mod common;

use simplex::program::{ProgramTrait, WitnessTrait};
use simplex::simplicityhl::elements::Sequence;
use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature};

use common::core::{Expect, fund, run_with_sequence};

use simplicityhl_std::artifacts::timelocks_test::TimelocksTestProgram;
use simplicityhl_std::artifacts::timelocks_test::derived_timelocks_test::{
    TimelocksTestArguments, TimelocksTestWitness,
};

// Dispatch indices — must match the `if_test_this_function(N, ..)` arms in
// simf/timelocks_test.simf.
enum FunctionToTest {
    EnforceRelativeDistance,
    EnforceRelativeDuration,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

// The `min_*` argument passed into the function under test in every case; only the
// *declared* (encoded) sequence value varies between test cases.
const MIN_DISTANCE: u16 = 5;
const MIN_DURATION: u16 = 5;

fn program() -> TimelocksTestProgram {
    TimelocksTestProgram::new(TimelocksTestArguments {})
}

fn build_witness(function: u8) -> TimelocksTestWitness {
    TimelocksTestWitness {
        function_index: function,
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

mod timelocks_test {
    use super::*;

    // ---------- shared: transaction version precondition ----------

    #[simplex::test]
    fn tx_version_below_2_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // `FinalTransaction` always builds a PSET-v2 transaction (tx version 2) and
        // doesn't expose a version setter, so `run_with_sequence`/`Signer::broadcast`
        // can't produce a version<2 transaction. This test drops one level lower: it
        // extracts the PSET itself, overrides the version field directly, then calls
        // `ProgramTrait::finalize` (the same local Simplicity execution that
        // `Signer::broadcast` runs internally before ever touching the network) instead
        // of going through `Signer::broadcast`, which would silently re-derive its own
        // PSET from `FinalTransaction` and ignore the override.
        //
        // `pst.global.tx_data.version` is the right PSET-v2 field for `elements =
        // 0.25.3` -- confirmed by this test passing.
        let program = program();
        let witness = build_witness(op(FunctionToTest::EnforceRelativeDistance));

        let script = fund(&context, &program)?;
        let utxos = context
            .get_default_provider()
            .fetch_scripthash_utxos(&script)?;

        let mut ft = FinalTransaction::new();
        // Plain `add_input` (not `add_program_input`): `extract_pst()` only reads
        // `partial_input`/outputs, so the program/witness don't need to be attached to
        // `ft` here -- we hand them to `finalize()` directly below.
        ft.add_input(PartialInput::new(utxos[0].clone()), RequiredSignature::None);

        let (mut pst, _secrets) = ft.extract_pst();
        pst.global.tx_data.version = 1;

        let witness_values = witness.build_witness();
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
            build_witness(op(FunctionToTest::EnforceRelativeDistance)),
            seq_disabled(),
            0,
            Expect::PrunedBranch,
        )
    }

    #[simplex::test]
    fn distance_wrong_variant_is_pruned(context: simplex::TestContext) -> anyhow::Result<()> {
        // Sequence declares a Duration (type flag set), but the function requires a
        // Distance -> unwrap_left panics on Right(_). 0 units is trivially BIP68-valid
        // (irrelevant here anyway, since this fails before broadcast).
        run_with_sequence(
            &context,
            program(),
            build_witness(op(FunctionToTest::EnforceRelativeDistance)),
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
            build_witness(op(FunctionToTest::EnforceRelativeDistance)),
            seq_distance(MIN_DISTANCE - 1),
            0,
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn distance_equal_minimum_succeeds(context: simplex::TestContext) -> anyhow::Result<()> {
        // Happy path: mine enough blocks that the declared distance is genuinely
        // satisfied on-chain, so the real BIP68 check at broadcast also passes.
        run_with_sequence(
            &context,
            program(),
            build_witness(op(FunctionToTest::EnforceRelativeDistance)),
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
            build_witness(op(FunctionToTest::EnforceRelativeDistance)),
            seq_distance(MIN_DISTANCE + 1),
            u64::from(MIN_DISTANCE + 1),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn distance_sufficient_declared_insufficient_real_is_rejected(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        // Declared distance satisfies MIN_DISTANCE, so local execution succeeds and the
        // transaction reaches the node. But we deliberately don't mine any extra blocks
        // first, so real elapsed distance since the funding confirmation is far short of
        // what's declared -> the node's own BIP68 check should reject the broadcast.
        // This is the test that actually proves the function's guarantee is backed by
        // consensus, not just by a value the spender is free to write into the tx.
        run_with_sequence(
            &context,
            program(),
            build_witness(op(FunctionToTest::EnforceRelativeDistance)),
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
            build_witness(op(FunctionToTest::EnforceRelativeDuration)),
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
            build_witness(op(FunctionToTest::EnforceRelativeDuration)),
            seq_distance(0),
            0,
            Expect::PrunedBranch,
        )
    }

    #[simplex::test]
    fn duration_below_minimum_fails(context: simplex::TestContext) -> anyhow::Result<()> {
        // NOTE: this is a "weak" version of the case (declared duration insufficient,
        // spend attempted at *some* real duration) rather than the full case (declared
        // duration insufficient, spend attempted at a real duration that's genuinely
        // sufficient for the *declared* value, isolating that the failure is really
        // about MIN_DURATION and not an artifact of insufficient real elapsed time).
        // Strengthening it that way would need median-time-past to have genuinely
        // advanced, which hits the same missing `setmocktime` capability as the happy
        // path above. It's a correctness non-issue either way -- this fails during
        // local Simplicity execution inside `Signer::broadcast`, before the transaction
        // is ever sent to the node, so real chain state can't affect the outcome here
        // regardless -- but it does mean this test doesn't positively demonstrate that
        // isolation the way `distance_below_minimum_fails` could (and doesn't yet).
        run_with_sequence(
            &context,
            program(),
            build_witness(op(FunctionToTest::EnforceRelativeDuration)),
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
        // Unlike the duration *happy* path, this needs no mocktime: "real MTP hasn't
        // advanced far enough yet" is simply the default state right after funding, not
        // something that has to be faked.
        run_with_sequence(
            &context,
            program(),
            build_witness(op(FunctionToTest::EnforceRelativeDuration)),
            seq_duration(MIN_DURATION),
            0,
            Expect::BroadcastRejected,
        )
    }

    // `duration_equal_minimum_succeeds` / `duration_above_minimum_succeeds` are
    // intentionally not included here: they require median-time-past to have genuinely
    // advanced by MIN_DURATION * 512 real seconds, which regtest can't be made to do
    // without a `setmocktime`-based helper that Simplex doesn't currently expose (see
    // `NetworkUtils`/`ElementsRpc` in `smplx/crates/test` and `smplx/crates/sdk`). Add
    // them once that lands.
}


