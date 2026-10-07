// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use secp256k1_zkp::Secp256k1;
use simplex::program::{Program, WitnessTrait};
use simplex::signer::Signer;
use simplex::simplicityhl::elements::{AssetId, Script, Txid};
use simplex::transaction::partial_input::IssuanceInput;
use simplex::transaction::{
    FinalTransaction, PartialInput, PartialOutput, ProgramInput, RequiredSignature, UTXO,
};

use crate::common::utxo_helper::{DEFAULT_SEND_AMOUNT, search_utxo_by_txid};

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
    amount_to_send: u64,
) -> anyhow::Result<Script> {
    let script = program.as_ref().get_script_pubkey(context.get_network());

    send_explicit(context.get_default_signer(), &script, amount_to_send)?;

    Ok(script)
}

/// Send sats to the specified script so it has a UTXO to spend.
pub fn send_explicit(signer: &Signer, to: &Script, amount_to_send: u64) -> anyhow::Result<Txid> {
    Ok(signer.send(to.clone(), amount_to_send)?.txid())
}

pub fn send_blinded(
    signer: &Signer,
    to: &Script,
    amount_to_send: u64,
    asset: AssetId,
) -> anyhow::Result<Txid> {
    let mut ft = FinalTransaction::new();

    ft.add_output(
        PartialOutput::new(to.clone(), amount_to_send, asset)
            .with_blinding_key(signer.get_blinding_public_key()),
    );

    Ok(signer.broadcast(&ft)?.txid())
}

/// Issue `amount_to_send` of a new asset to the specified script, so the UTXO holds an asset that
/// nothing else in the wallet has. Returns the funding txid and the new asset id.
pub fn send_issued(
    context: &simplex::TestContext,
    to: &Script,
    amount_to_send: u64,
) -> anyhow::Result<(Txid, AssetId)> {
    let signer = context.get_default_signer();
    // The largest UTXO, so the issuance never spends one of the small UTXOs a test created.
    let funding = signer
        .get_utxos_asset(context.get_network().policy_asset())?
        .into_iter()
        .max_by_key(UTXO::amount)
        .ok_or_else(|| anyhow::anyhow!("no policy asset UTXO to attach the issuance to"))?;

    let mut ft = FinalTransaction::new();
    let issued = ft.add_issuance_input(
        PartialInput::new(funding),
        IssuanceInput::new_issuance(amount_to_send, 0, rand::random()),
        RequiredSignature::NativeEcdsa,
    );
    ft.add_output(PartialOutput::new(
        to.clone(),
        amount_to_send,
        issued.asset_id,
    ));

    Ok((signer.broadcast(&ft)?.txid(), issued.asset_id))
}

/// Construct the funded UTXO with `witness`.
pub fn construct_final_tx<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    data: Option<&[u8]>,
    txids: Vec<Txid>,
) -> anyhow::Result<FinalTransaction>
where
    W: WitnessTrait + 'static,
{
    let signer = context.get_default_signer();
    let script_to_send_to = signer.get_address().script_pubkey();
    let policy_asset = context.get_network().policy_asset();

    // Every output amount is one less than the corresponding input amount.
    // The signer handles leftovers for the policy asset, but for other assets, we need to handle it here.
    let mut leftover_asset = policy_asset;

    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let mut program_utxo = if txids.is_empty() {
        utxos[0].clone()
    } else {
        search_utxo_by_txid(&txids[0], &utxos)?
    };

    let first_input_confidential = program_utxo.txout.value.is_confidential();

    if first_input_confidential {
        program_utxo = unblind(signer, program_utxo)?;
    }

    let mut ft = FinalTransaction::new();

    ft.add_program_input(
        PartialInput::new(program_utxo.clone()),
        ProgramInput::new(Box::new(program.as_ref().clone()), Box::new(witness)),
        RequiredSignature::None,
    );

    // it is not an op_return path
    if data.unwrap_or_default().is_empty() {
        let mut output = PartialOutput::new(
            script_to_send_to.clone(),
            // amounts in input and output on the same index should be different for the test purposes
            program_utxo.amount() - 1,
            program_utxo.asset(),
        );

        if first_input_confidential {
            output = output.with_blinding_key(signer.get_blinding_public_key())
        }

        ft.add_output(output);

        if program_utxo.asset() != policy_asset {
            leftover_asset = program_utxo.asset();
        }
    }

    if txids.len() > 1 {
        let unblinded_utxos = signer.get_utxos()?;

        for txid in txids.iter().skip(1) {
            let utxo = search_utxo_by_txid(txid, &unblinded_utxos)?;

            if !utxo.txout.value.is_confidential() {
                ft.add_input(PartialInput::new(utxo.clone()), RequiredSignature::None);

                ft.add_output(PartialOutput::new(
                    script_to_send_to.clone(),
                    utxo.explicit_amount() - 1,
                    utxo.explicit_asset(),
                ));

                if utxo.explicit_asset() != policy_asset {
                    leftover_asset = utxo.asset();
                }
                // filtering out fund transaction
            } else {
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

    if leftover_asset != policy_asset {
        ft.add_output(PartialOutput::new(
            script_to_send_to.clone(),
            1,
            leftover_asset,
        ));
    }

    if let Some(data) = data {
        ft.add_output(PartialOutput::new_metadata(data))
    };

    Ok(ft)
}

fn unblind(signer: &Signer, mut utxo: UTXO) -> anyhow::Result<UTXO> {
    let secp = Secp256k1::new();
    let secrets = utxo
        .txout
        .unblind(&secp, signer.get_blinding_private_key().inner)?;
    utxo.secrets = Some(secrets);
    Ok(utxo)
}

/// Spend the funded UTXO with `witness`. Return the broadcast result.
pub fn spend<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    script: &Script,
    witness: W,
    data: Option<&[u8]>,
    txids: Vec<Txid>,
) -> anyhow::Result<String>
where
    W: WitnessTrait + 'static,
{
    let ft = construct_final_tx(context, program, script, witness, data, txids)?;

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
    let result = spend(context, &program, &script, witness, None, Vec::new());

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
    let result = spend(context, &program, &script, witness, Some(data), Vec::new());

    assert_error_msg(result, expect)
}

/// Spend with additional inputs and outputs + assert the outcome.
pub fn run_with_inputs_outputs<W>(
    context: &simplex::TestContext,
    program: impl AsRef<Program>,
    witness: W,
    expect: Expect,
    txids: Vec<Txid>,
) -> anyhow::Result<()>
where
    W: WitnessTrait + 'static,
{
    let script = program.as_ref().get_script_pubkey(context.get_network());

    let result = spend(context, &program, &script, witness, None, txids);

    assert_error_msg(result, expect)
}
