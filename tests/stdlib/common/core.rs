// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use secp256k1_zkp::Secp256k1;
use simplex::program::{Program, WitnessTrait};
use simplex::signer::{Signer, SignerError};
use simplex::simplicityhl::elements::{AssetId, Script};
use simplex::transaction::{
    FinalTransaction, PartialInput, PartialOutput, ProgramInput, RequiredSignature, TxReceipt,
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

const DEFAULT_SEND_AMOUNT: u64 = 50;

/// Send sats to the program's script so it has a UTXO to spend.
pub fn fund(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    amount_to_send: u64,
) -> anyhow::Result<Script> {
    let script = program.as_ref().get_script_pubkey(context.get_network());

    context
        .get_default_signer()
        .send(script.clone(), amount_to_send)?;

    Ok(script)
}

/// Send sats to the specified script so it has a UTXO to spend.
pub fn send_explicit(signer: &Signer, to: &Script, amount_to_send: u64) -> anyhow::Result<()> {
    signer.send(to.clone(), amount_to_send)?;

    Ok(())
}

pub fn send_with_blinding_return_txid<'a>(
    signer: &'a Signer,
    to: &Script,
    amount_to_send: u64,
    asset: AssetId,
) -> Result<TxReceipt<'a>, SignerError> {
    let mut ft = FinalTransaction::new();

    ft.add_output(
        PartialOutput::new(to.clone(), amount_to_send, asset)
            .with_blinding_key(signer.get_blinding_public_key()),
    );

    signer.broadcast(&ft)
}

/// Construct the funded UTXO with `witness`.
pub fn construct_final_tx<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    data: Option<&[u8]>,
    index: u32,
    is_explicit: bool,
) -> anyhow::Result<FinalTransaction>
where
    W: WitnessTrait + 'static,
{
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let mut utxo = utxos[0].clone();
    let signer = context.get_default_signer();

    // unblinding utxo with the program
    if index == 0 && !is_explicit {
        let secp = Secp256k1::new();
        let secrets = utxos[0]
            .txout
            .unblind(&secp, signer.get_blinding_private_key().inner)?;
        utxo.secrets = Some(secrets);
    }

    let mut ft = FinalTransaction::new();

    ft.add_program_input(
        PartialInput::new(utxo.clone()),
        ProgramInput::new(Box::new(program.as_ref().clone()), Box::new(witness)),
        RequiredSignature::None,
    );

    if data.unwrap_or_default().is_empty() {
        // it is not an op_return path
        ft.add_output(PartialOutput::new(
            Script::new(),
            // amounts in unput and output on the same index should be different for the test purposes
            utxo.amount() - 1,
            utxo.asset(),
        ));
    }

    if index > 0 {
        let unblinded_utxos = signer.get_utxos()?;
        let script_to_send_to = signer.get_address().script_pubkey();

        assert!(
            unblinded_utxos.len() == index as usize + 1,
            "Not enough utxo's"
        ); // unblinded_utxos contain an initial fund utxo, it won't be in our tx

        for utxo in &unblinded_utxos {
            // todo order utxos
            if !utxo.txout.nonce.is_confidential() {
                ft.add_input(PartialInput::new(utxo.clone()), RequiredSignature::None);

                ft.add_output(PartialOutput::new(
                    script_to_send_to.clone(),
                    utxo.explicit_amount() - 1,
                    utxo.explicit_asset(),
                ));
                // filtering out fund transaction
            } else if utxo.unblinded_amount() < 99999999 {
                ft.add_input(PartialInput::new(utxo.clone()), RequiredSignature::None);

                ft.add_output(
                    PartialOutput::new(
                        script_to_send_to.clone(),
                        utxo.unblinded_amount() - 1,
                        utxo.unblinded_asset(),
                    )
                    .with_blinding_key(signer.get_blinding_public_key()),
                );
            }
        }
    }

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
    index: u32,
    is_explicit: bool,
) -> anyhow::Result<String>
where
    W: WitnessTrait + 'static,
{
    let ft = construct_final_tx(context, program, script, witness, data, index, is_explicit)?;

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
    let script = fund(context, &program, DEFAULT_SEND_AMOUNT)?;
    let result = spend(context, &program, &script, witness, None, 0, true);

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
    let script = fund(context, &program, DEFAULT_SEND_AMOUNT)?;
    let result = spend(context, &program, &script, witness, Some(data), 0, true);

    assert_error_msg(result, expect)
}

/// Spend with additional inputs and outputs + assert the outcome.
pub fn run_w_inputs_outputs<W>(
    context: &simplex::TestContext,
    program: impl AsRef<Program>,
    witness: W,
    expect: Expect,
    index: u32,
    is_explicit: bool,
) -> anyhow::Result<()>
where
    W: WitnessTrait + 'static,
{
    let script = program.as_ref().get_script_pubkey(context.get_network());

    let result = spend(
        context,
        &program,
        &script,
        witness,
        None,
        index,
        is_explicit,
    );

    assert_error_msg(result, expect)
}
