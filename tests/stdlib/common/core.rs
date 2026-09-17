// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use simplex::program::Program;
use simplex::simplicityhl::WitnessValues;
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
    W: Into<WitnessValues> + 'static,
{
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let mut ft = FinalTransaction::new();
    ft.add_program_input(
        PartialInput::new(utxos[0].clone()),
        ProgramInput::new(Box::new(program.as_ref().clone()), witness),
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
    W: Into<WitnessValues> + 'static,
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
    W: Into<WitnessValues> + 'static,
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
    W: Into<WitnessValues> + 'static,
{
    let script = fund(context, &program)?;
    let result = spend(context, &program, &script, witness, Some(data));

    assert_error_msg(result, expect)
}

use simplex::fuzz::core::FuzzContext;
use simplex::fuzz::{ProgramCheck, ProgramExecResult};
use simplex::program::ProgramError;
use simplex::simplicityhl::elements::pset::PartiallySignedTransaction;
use simplex::simplicityhl::simplicity::bit_machine::ExecutionError;

/// Checks that a fuzzed program produces the exact execution outcome expected
/// by the test case.
pub struct FuzzExecutionCheck {
    test_name: &'static str,
    expect: Expect,
}

impl FuzzExecutionCheck {
    pub const fn new(test_name: &'static str, expect: Expect) -> Self {
        Self { test_name, expect }
    }
}

impl ProgramCheck for FuzzExecutionCheck {
    fn call(
        &self,
        _context: &FuzzContext,
        _transaction: &PartiallySignedTransaction,
        _arguments: &simplex::simplicityhl::Arguments,
        _witness: &WitnessValues,
        _input_index: usize,
        program_exec_result: ProgramExecResult,
    ) -> Result<(), String> {
        match (self.expect, program_exec_result) {
            (Expect::Ok, Ok(_)) => Ok(()),
            (Expect::AssertFailed, Err(ProgramError::Pruning(ExecutionError::JetFailed(_)))) => {
                Ok(())
            }
            (
                Expect::PrunedBranch,
                Err(ProgramError::Pruning(ExecutionError::ReachedPrunedBranch(_))),
            ) => Ok(()),
            (expect, Ok(_)) => Err(format!(
                "{} unexpectedly succeeded; expected {}",
                self.test_name,
                expected_outcome(expect)
            )),
            (expect, Err(error)) => Err(format!(
                "{} failed with {error}; expected {}",
                self.test_name,
                expected_outcome(expect)
            )),
        }
    }
}

fn expected_outcome(expect: Expect) -> &'static str {
    match expect {
        Expect::Ok => "a successful execution",
        Expect::AssertFailed => "a jet failure from assert!",
        Expect::PrunedBranch => "a reached pruned branch",
    }
}
