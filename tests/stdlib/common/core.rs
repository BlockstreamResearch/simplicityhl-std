// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use simplex::program::{Program, WitnessTrait};
use simplex::simplicityhl::elements::Script;
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
}

impl Expect {
    /// The exact broadcast error message for a failing expectation (`None` for `Ok`).
    fn error_message(self) -> Option<&'static str> {
        match self {
            Expect::Ok => None,
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
    match expect.error_message() {
        None => {
            result?;
        }
        Some(expected) => {
            let err = result
                .expect_err("expected the spend to fail, but it succeeded")
                .to_string();
            assert!(err.contains(expected));
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
