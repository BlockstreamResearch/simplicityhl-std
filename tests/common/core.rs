// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use simplex::program::{Program, WitnessTrait};
use simplex::simplicityhl::elements::{Script, Sequence};
use simplex::transaction::{
    FinalTransaction, PartialInput, PartialOutput, ProgramInput, RequiredSignature,
};

#[derive(Clone, Copy)]
pub enum Expect {
    /// The spend succeeds.
    Ok,
    /// A failed `assert!` in the contract.
    AssertFailed,
    /// Execution reached a pruned branch (e.g. `unwrap(None)`, a `safe_*` overflow).
    PrunedBranch,
    /// Local Simplicity execution succeeds, but the node itself rejects the finished
    /// transaction (e.g. a BIP68 relative-locktime declared in nSequence that hasn't
    /// actually been satisfied on-chain yet). Unlike `AssertFailed`/`PrunedBranch`, which
    /// fail during local execution inside `Signer::broadcast` before the transaction is
    /// ever sent anywhere, this only fires once local execution has already succeeded
    /// and the tx reaches the node's own mempool-acceptance checks.
    BroadcastRejected,
}

impl Expect {
    /// The exact broadcast error message for a local-execution failure (`None` for
    /// `Ok`/`BroadcastRejected`, which are handled separately in `assert_error_msg`).
    fn error_message(self) -> Option<&'static str> {
        match self {
            Expect::Ok | Expect::BroadcastRejected => None,
            Expect::AssertFailed => Some("Failed to prune program: Jet failed during execution"),
            Expect::PrunedBranch => {
                Some("Failed to prune program: Execution reached a pruned branch")
            }
        }
    }
}
/// Send sats to the program's script so it has a UTXO to spend.
pub fn fund(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
) -> anyhow::Result<Script> {
    let script = program.as_ref().get_script_pubkey(context.get_network());

    context.get_default_signer().send(script.clone(), 50)?;

    Ok(script)
}

/// Construct the funded UTXO with `witness`.
pub fn construct_final_tx<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    data: Option<&[u8]>,
) -> anyhow::Result<FinalTransaction>
where
    W: WitnessTrait + 'static,
{
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let mut ft = FinalTransaction::new();
    ft.add_program_input(
        PartialInput::new(utxos[0].clone()),
        ProgramInput::new(Box::new(program.as_ref().clone()), Box::new(witness)),
        RequiredSignature::None,
    );

    if let Some(data) = data {
        ft.add_output(PartialOutput::new_metadata(data))
    };

    Ok(ft)
}

/// Spend the funded UTXO with `witness`. Return the broadcast result.
pub fn spend<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    data: Option<&[u8]>,
) -> anyhow::Result<String>
where
    W: WitnessTrait + 'static,
{
    let ft = construct_final_tx(context, program, script, witness, data)?;

    Ok(context.get_default_signer().broadcast(&ft)?.to_string())
}

/// Assert that the test result is as expected.
pub fn assert_error_msg(
    result: Result<String, anyhow::Error>,
    expect: Expect,
) -> anyhow::Result<()> {
    match expect {
        Expect::Ok => {
            result?;
        }
        Expect::BroadcastRejected => {
            // Confirmed against a live regtest node: elementsd (inheriting Bitcoin
            // Core's mempool policy) rejects a transaction whose declared BIP68
            // relative-locktime hasn't actually been satisfied on-chain with
            // `sendrawtransaction RPC error -26: non-BIP68-final`, regardless of
            // whether the unmet lock was a distance or a duration.
            let err = result
                .expect_err("expected the node to reject the broadcast, but it succeeded")
                .to_string();
            assert!(
                err.contains("non-BIP68-final"),
                "expected a `non-BIP68-final` rejection, got: {err}"
            );
        }
        Expect::AssertFailed | Expect::PrunedBranch => {
            let expected = expect
                .error_message()
                .expect("AssertFailed/PrunedBranch always have a fixed error message");
            let err = result
                .expect_err("expected the spend to fail, but it succeeded")
                .to_string();
            assert!(err.contains(expected), "expected `{expected}`, got: {err}");
        }
    };

    Ok(())
}

/// Fund + spend + assert the outcome.
pub fn run<W>(
    context: &simplex::TestContext,
    program: impl AsRef<Program>,
    witness: W,
    expect: Expect,
) -> anyhow::Result<()>
where
    W: WitnessTrait + 'static,
{
    let script = fund(context, &program)?;
    let result = spend(context, &program, &script, witness, None);

    assert_error_msg(result, expect)
}

/// Fund + spend + assert the outcome.
/// Tx has OP_RETURN data metadata output
pub fn run_with_op_return<W>(
    context: &simplex::TestContext,
    program: impl AsRef<Program>,
    witness: W,
    expect: Expect,
    data: &[u8],
) -> anyhow::Result<()>
where
    W: WitnessTrait + 'static,
{
    let script = fund(context, &program)?;
    let result = spend(context, &program, &script, witness, Some(data));

    assert_error_msg(result, expect)
}

// Add `Sequence` to the existing import:
// use simplex::simplicityhl::elements::{Script, Sequence};

/// Construct the funded UTXO with `witness`, spent under a caller-chosen `sequence`
/// (nSequence) instead of the default (relative-timelock-disabled) value.
pub fn construct_final_tx_with_sequence<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    sequence: Sequence,
) -> anyhow::Result<FinalTransaction>
where
    W: WitnessTrait + 'static,
{
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let mut ft = FinalTransaction::new();
    ft.add_program_input(
        PartialInput::new(utxos[0].clone()).with_sequence(sequence),
        ProgramInput::new(Box::new(program.as_ref().clone()), Box::new(witness)),
        RequiredSignature::None,
    );

    Ok(ft)
}

/// Spend the funded UTXO with `witness` under a caller-chosen `sequence`. Return the
/// broadcast result.
pub fn spend_with_sequence<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    sequence: Sequence,
) -> anyhow::Result<String>
where
    W: WitnessTrait + 'static,
{
    let ft = construct_final_tx_with_sequence(context, program, script, witness, sequence)?;

    Ok(context.get_default_signer().broadcast(&ft)?.to_string())
}

/// Fund + spend + assert the outcome, using a custom relative-locktime `sequence`.
///
/// `blocks_to_mine` mines that many *additional* blocks (beyond the 1 confirmation the
/// funding tx already has) before broadcasting, so a real BIP68 relative-locktime
/// requirement encoded in `sequence` is genuinely satisfied on-chain. It matters only for
/// `Expect::Ok` cases: `Expect::AssertFailed`/`Expect::PrunedBranch` cases fail inside
/// local Simplicity execution during `Signer::broadcast`'s witness-finalization step,
/// before the transaction is ever sent to the node, so real chain state never comes into
/// play for them — 0 is always correct there.
pub fn run_with_sequence<W>(
    context: &simplex::TestContext,
    program: impl AsRef<Program>,
    witness: W,
    sequence: Sequence,
    blocks_to_mine: u64,
    expect: Expect,
) -> anyhow::Result<()>
where
    W: WitnessTrait + 'static,
{
    let script = fund(context, &program)?;

    if blocks_to_mine > 0 {
        let target = context.get_default_provider().fetch_tip_height()? as u64 + blocks_to_mine;
        context.get_network_utils().mine_until_height(target)?;
    }

    let result = spend_with_sequence(context, &program, &script, witness, sequence);

    assert_error_msg(result, expect)
}
