// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use secp256k1_zkp::Secp256k1;
use simplex::program::{Program, WitnessTrait};
use simplex::signer::Signer;
use simplex::simplicityhl::elements::bitcoin::PublicKey;
use simplex::simplicityhl::elements::{AssetId, Script, Txid};
use simplex::transaction::partial_input::IssuanceInput;
use simplex::transaction::{
    FinalTransaction, PartialInput, PartialOutput, ProgramInput, RequiredSignature, UTXO,
};
use simplicityhl_std::artifacts::tests::mock::MockProgram;
use simplicityhl_std::artifacts::tests::mock::derived_mock::MockWitness;

use crate::common::utxo_helper::{DEFAULT_SEND_AMOUNT, search_utxo_by_txid};

#[derive(Clone, Copy)]
pub enum Expect {
    /// The spend succeeds.
    Ok,
    /// A failed `assert!` in the contract.
    AssertFailed,
    /// Execution reached a pruned branch (e.g. `unwrap(None)`).
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

/// Send sats to the specified script confidentialy.
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
/// nothing else in the wallet has.
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
    witness: W,
    data: Option<&[u8]>,
    txids: Vec<Txid>,
    program_for_test: Option<&impl AsRef<Program>>,
) -> anyhow::Result<FinalTransaction>
where
    W: WitnessTrait + 'static,
{
    let signer = context.get_default_signer();
    let network = context.get_network();
    let policy_asset = network.policy_asset();

    let default_script = signer.get_address().script_pubkey();
    let program_script = &program.as_ref().get_script_pubkey(network);
    let mut leftover_asset: Option<AssetId> = None;

    let mut ft = FinalTransaction::new();
    let program_utxo = prepare_utxo(context, program_script, txids.first())?;

    ft.add_program_input(
        PartialInput::new(program_utxo.clone()),
        ProgramInput::new(Box::new(program.as_ref().clone()), Box::new(witness)),
        RequiredSignature::None,
    );

    if data.unwrap_or_default().is_empty() {
        (ft, leftover_asset) = add_mirror_output(
            ft,
            &program_utxo,
            signer.get_blinding_public_key(),
            policy_asset,
        );
    }

    if txids.len() > 1 {
        let unblinded_utxos = signer.get_utxos()?;
        let last = txids.len() - 1;

        for txid in &txids[1..last] {
            let utxo = search_utxo_by_txid(txid, &unblinded_utxos)?;

            ft.add_input(PartialInput::new(utxo.clone()), RequiredSignature::None);

            (ft, _) = add_mirror_output(ft, &utxo, signer.get_blinding_public_key(), policy_asset);
        }

        let last_utxo = match program_for_test {
            Some(program_for_test) => {
                let last_script = &program_for_test.as_ref().get_script_pubkey(network);
                let last_utxo = prepare_utxo(context, last_script, txids.last())?;

                ft.add_program_input(
                    PartialInput::new(last_utxo.clone()),
                    ProgramInput::new(
                        Box::new(program_for_test.as_ref().clone()),
                        Box::new(MockWitness {}),
                    ),
                    RequiredSignature::None,
                );

                last_utxo
            }
            None => {
                let last_utxo = search_utxo_by_txid(txids.last().unwrap(), &unblinded_utxos)?;

                ft.add_input(
                    PartialInput::new(last_utxo.clone()),
                    RequiredSignature::None,
                );

                last_utxo
            }
        };

        (ft, leftover_asset) = add_mirror_output(
            ft,
            &last_utxo,
            signer.get_blinding_public_key(),
            policy_asset,
        );
    }

    if let Some(leftover_asset) = leftover_asset {
        ft.add_output(PartialOutput::new(
            default_script.clone(),
            1,
            leftover_asset,
        ));
    }

    if let Some(data) = data {
        ft.add_output(PartialOutput::new_metadata(data))
    };

    Ok(ft)
}

/// Fetch the UTXO for script, unblind if confidential.
fn prepare_utxo(
    context: &simplex::TestContext,
    script: &Script,
    txid: Option<&Txid>,
) -> anyhow::Result<UTXO> {
    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let utxo = match txid {
        Some(txid) => search_utxo_by_txid(txid, &utxos)?,
        None => utxos
            .into_iter()
            .next()
            .expect("Missing UTXO for the script"),
    };

    if utxo.txout.value.is_confidential() {
        Ok(unblind(context.get_default_signer(), utxo)?)
    } else {
        Ok(utxo)
    }
}

/// Unblind the utxo.
fn unblind(signer: &Signer, mut utxo: UTXO) -> anyhow::Result<UTXO> {
    let secp = Secp256k1::new();
    let secrets = utxo
        .txout
        .unblind(&secp, signer.get_blinding_private_key().inner)?;
    utxo.secrets = Some(secrets);
    Ok(utxo)
}

// Add an output mirroring utxo's asset with amount - 1 and the same script.
fn add_mirror_output(
    mut ft: FinalTransaction,
    utxo: &UTXO,
    binding_key: PublicKey,
    policy_asset: AssetId,
) -> (FinalTransaction, Option<AssetId>) {
    let mut output = PartialOutput::new(
        utxo.txout.script_pubkey.clone(),
        // amounts in input and output on the same index should be different for the test
        utxo.amount() - 1,
        utxo.asset(),
    );

    if utxo.txout.value.is_confidential() {
        output = output.with_blinding_key(binding_key)
    }

    ft.add_output(output);

    let leftover_asset = if utxo.asset() != policy_asset {
        Some(utxo.asset())
    } else {
        None
    };

    (ft, leftover_asset)
}

/// Spend the funded UTXO with `witness`. Return the broadcast result.
pub fn spend<W>(
    context: &simplex::TestContext,
    program: &impl AsRef<Program>,
    witness: W,
    data: Option<&[u8]>,
    txids: Vec<Txid>,
    program_for_test: Option<&impl AsRef<Program>>,
) -> anyhow::Result<String>
where
    W: WitnessTrait + 'static,
{
    let ft = construct_final_tx(context, program, witness, data, txids, program_for_test)?;

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
    fund(context, &program, DEFAULT_SEND_AMOUNT)?;
    let result = spend(
        context,
        &program,
        witness,
        None,
        Vec::new(),
        None::<&MockProgram>,
    );

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
    fund(context, &program, DEFAULT_SEND_AMOUNT)?;
    let result = spend(
        context,
        &program,
        witness,
        Some(data),
        Vec::new(),
        None::<&MockProgram>,
    );

    assert_error_msg(result, expect)
}

/// Spend with additional inputs and outputs + assert the outcome.
pub fn run_with_inputs_outputs<W>(
    context: &simplex::TestContext,
    program: impl AsRef<Program>,
    witness: W,
    expect: Expect,
    txids: Vec<Txid>,
    program_for_test: Option<&impl AsRef<Program>>,
) -> anyhow::Result<()>
where
    W: WitnessTrait + 'static,
{
    let result = spend(context, &program, witness, None, txids, program_for_test);

    assert_error_msg(result, expect)
}
