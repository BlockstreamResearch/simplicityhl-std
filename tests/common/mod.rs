use simplex::program::{Program, WitnessTrait};
use simplex::simplicityhl::elements::Script;
use simplex::transaction::{FinalTransaction, PartialInput, ProgramInput, RequiredSignature};

pub mod uint;

// ---------------------------------------------------------------------------
// Spend plumbing (shared by every test, of every category)
// ---------------------------------------------------------------------------

pub enum Expect {
    Ok,
    PrunedFail, // safe_* overflow, div-by-zero, etc.
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

/// Spend the funded UTXO with `witness`. Returns the broadcast result.
pub fn spend<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
) -> anyhow::Result<String>
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

    Ok(context.get_default_signer().broadcast(&ft)?.to_string())
}

/// Fund + spend + assert the outcome, in one call.
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
    let result = spend(context, &program, &script, witness);

    match expect {
        Expect::Ok => {
            result?;
        }
        Expect::PrunedFail => {
            assert!(
                result.is_err(),
                "expected a pruned-branch failure, but it succeeded"
            );
            let err = result.unwrap_err().to_string();
            assert!(err.contains("Failed to prune program: Execution reached a pruned branch"));
        }
    }
    Ok(())
}
