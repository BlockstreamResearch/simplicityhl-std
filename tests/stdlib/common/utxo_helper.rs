use simplex::simplicityhl::elements::AssetId;
use simplex::simplicityhl::elements::OutPoint;
use simplex::simplicityhl::elements::Script;
use simplex::simplicityhl::elements::Txid;
use simplex::simplicityhl::elements::pset::serialize::Serialize;

use simplex::program::Program;
use simplex::transaction::UTXO;

use primitive_types::U256;
use rand::Rng;

use crate::common::core::tweak_script;
use crate::common::core::{send_blinded, send_explicit, send_issued};
use crate::common::helper::generate_u256;

pub const DEFAULT_SEND_AMOUNT: u64 = 50;

/// The funding transaction holds around 10^16 satoshis, so 10_000 is used
/// as the limit to ensure there are enough satoshis for all the inputs and outputs.
pub fn random_expected_amount() -> u64 {
    rand::thread_rng().gen_range(100..=10_000)
}

pub struct ConfidentialAssetId {
    pub parity_bit: u8,
    pub asset_id: [u8; 32],
}
pub struct ConfidentialAmount {
    pub parity_bit: u8,
    pub amount: [u8; 32],
}

pub type ConfidentialAsset = (ConfidentialAssetId, ConfidentialAmount);

pub fn create_utxos(
    context: &simplex::TestContext,
    index: u32,
    is_explicit: bool,
    is_input: bool,
    expected_amount: u64,
    last_script: &Script,
    program: impl AsRef<Program>,
) -> anyhow::Result<(UTXO, Vec<Txid>)> {
    let signer = context.get_default_signer();
    let pubkey_script = &signer.get_address().script_pubkey();
    let program_script = &program.as_ref().get_script_pubkey(context.get_network());

    let mut txids = Vec::new();

    for i in 0..=index {
        let script = if i == 0 {
            program_script
        } else if i == index {
            last_script
        } else {
            pubkey_script
        };

        let mut amount_to_send = match index == i {
            true => expected_amount,
            false => DEFAULT_SEND_AMOUNT,
        };
        // while testing an input amount, input utxo has `amount` and output has `amount - 1`;
        // if an output amount is tested, input utxo has `amount + 1` and output has `amount`.
        if !is_input {
            amount_to_send += 1;
        }

        if index == i && !is_explicit {
            txids.push(send_blinded(
                signer,
                script,
                amount_to_send,
                context.get_network().policy_asset(),
            )?)
        } else {
            txids.push(send_explicit(signer, script, amount_to_send)?);
        }
    }

    let script = match index == 0 {
        true => program_script,
        false => last_script,
    };

    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    let last_utxo = search_utxo_by_txid(txids.last().unwrap(), &utxos)?;

    Ok((last_utxo, txids))
}

pub fn create_utxos_and_return_conf_assets(
    context: &simplex::TestContext,
    index: u32,
    is_explicit: bool,
    is_input: bool,
    expected_amount: u64,
    program: impl AsRef<Program>,
) -> anyhow::Result<(Option<ConfidentialAsset>, Vec<Txid>)> {
    let last_script = &context.get_default_signer().get_address().script_pubkey();

    let (last_utxo, txids) = create_utxos(
        context,
        index,
        is_explicit,
        is_input,
        expected_amount,
        last_script,
        program,
    )?;

    match is_explicit {
        true => Ok((None, txids)),
        false => Ok((Some(get_asset_id_amount_from_conf_utxo(last_utxo)?), txids)),
    }
}

pub fn create_utxos_and_return_script_hash(
    context: &simplex::TestContext,
    index: u32,
    is_input: bool,
    last_script: &Script,
    program: impl AsRef<Program>,
) -> anyhow::Result<([u8; 32], Vec<Txid>)> {
    let (last_utxo, txids) = create_utxos(
        context,
        index,
        true,
        is_input,
        DEFAULT_SEND_AMOUNT,
        last_script,
        program,
    )?;

    // when testing an input amount, we expect the actual script;
    // the output script is tweaked to differ from the input script
    let script = match is_input {
        true => last_utxo.txout.script_pubkey,
        false => tweak_script(&last_utxo.txout.script_pubkey),
    };

    let hash = script.wscript_hash();

    Ok((hash.to_byte_array(), txids))
}

pub fn create_utxos_with_new_asset(
    context: &simplex::TestContext,
    index: u32,
    is_input: bool,
    expected_amount: u64,
    program: impl AsRef<Program>,
) -> anyhow::Result<([u8; 32], Vec<Txid>)> {
    let signer = context.get_default_signer();
    let pubkey_script = &signer.get_address().script_pubkey();
    let program_script = &program.as_ref().get_script_pubkey(context.get_network());

    let mut txids = Vec::new();
    let mut new_asset = None::<AssetId>;

    for i in 0..=index {
        let script = match i == 0 {
            true => program_script,
            false => pubkey_script,
        };
        let mut amount_to_send = match index == i {
            true => expected_amount,
            false => DEFAULT_SEND_AMOUNT,
        };
        // same convention as in `create_utxos`
        if !is_input {
            amount_to_send += 1;
        }

        if index == i {
            let (txid, asset) = send_issued(context, script, amount_to_send)?;
            txids.push(txid);
            new_asset = Some(asset);
        } else {
            txids.push(send_explicit(signer, script, amount_to_send)?);
        }
    }

    Ok((new_asset.unwrap().to_byte_array(), txids))
}

fn get_asset_id_amount_from_conf_utxo(conf_utxo: UTXO) -> anyhow::Result<ConfidentialAsset> {
    let confidential_asset = conf_utxo.txout.asset.serialize(); // 0x0a | 0x0b
    let confidential_amount = conf_utxo.txout.value.serialize(); // 0x08 | 0x09

    // 0x0a = 00001010 & 1 = 0
    // 0x0b = 00001011 & 1 = 1
    // 0x08 = 00001000 & 1 = 0
    // 0x09 = 00001001 & 1 = 1
    let (asset_parity_bit, asset_id) = decode_point(confidential_asset.as_slice().try_into()?);
    let (amount_parity_bit, amount) = decode_point(confidential_amount.as_slice().try_into()?);

    Ok((
        ConfidentialAssetId {
            parity_bit: asset_parity_bit,
            asset_id,
        },
        ConfidentialAmount {
            parity_bit: amount_parity_bit,
            amount,
        },
    ))
}

pub fn search_utxo_by_txid(txid: &Txid, utxos: &[UTXO]) -> anyhow::Result<UTXO> {
    let outpoint = OutPoint {
        txid: *txid,
        vout: 0,
    };

    utxos
        .iter()
        .find(|utxo| utxo.outpoint == outpoint)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing utxo with id: {txid}"))
}

pub fn generate_different_amounts() -> (u64, u64) {
    let some_u = random_expected_amount();
    let mut other_u = random_expected_amount();

    while other_u == some_u {
        other_u = random_expected_amount();
    }

    (some_u, other_u)
}

pub fn generate_random_confidential() -> (u8, [u8; 32]) {
    let parity_bit = rand::thread_rng().gen_range(0..=1);
    let asset = generate_u256(U256::zero(), U256::MAX).to_big_endian();

    (parity_bit, asset)
}

/// Decodes a 33-byte point encoding into its parity bit and x coordinate.
pub fn decode_point(data: [u8; 33]) -> (u8, [u8; 32]) {
    let mut coordinate = [0; 32];
    coordinate.copy_from_slice(&data[1..]);
    (data[0] & 1, coordinate)
}

pub mod fuzz {
    use simplex::simplicityhl::elements::confidential::{
        Asset, AssetBlindingFactor, Value, ValueBlindingFactor,
    };
    use simplex::simplicityhl::elements::secp256k1_zkp::Secp256k1;
    use simplex::simplicityhl::elements::{AssetId, OutPoint, Script, TxOutSecrets, Txid};

    use simplex::transaction::{FinalTransaction, PartialInput, PartialOutput, UTXO};
    use simplex::{FuzzMode, TestContext};

    pub const MAX_UTXO_INDEX: u32 = 20;
    pub const FUZZ_UTXO_COUNT: u32 = MAX_UTXO_INDEX + 1;

    /// Builds a balanced explicit transaction with distinct UTXOs.
    pub fn create_explicit_transaction() -> FinalTransaction {
        let mut transaction = FinalTransaction::new();

        for index in 0..FUZZ_UTXO_COUNT {
            let amount = u64::from(index + 1) * 1_000;
            let tag = index as u8 + 1;
            let asset = AssetId::from_byte_array([tag; 32]);
            let input = UTXO::explicit(
                OutPoint::new(Txid::from_byte_array([1; 32]), index),
                Script::from(vec![0x51, tag]),
                amount,
                asset,
            );
            let output = PartialOutput::new(Script::from(vec![0x52, tag]), amount, asset);

            transaction.add_input(PartialInput::new(input), None);
            transaction.add_output(output);
        }

        // Keep the transaction balanced while giving each position a different output.
        transaction.outputs_mut().reverse();
        transaction
    }

    /// Builds a balanced confidential transaction with distinct UTXOs.
    pub fn create_confidential_transaction(context: &TestContext<FuzzMode>) -> FinalTransaction {
        let mut transaction = FinalTransaction::new();
        let signer = context.get_default_signer();
        let secp = Secp256k1::new();
        let asset_blinder = AssetBlindingFactor::from_slice(&[1; 32]).unwrap();
        let value_blinder = ValueBlindingFactor::from_slice(&[2; 32]).unwrap();

        for index in 0..FUZZ_UTXO_COUNT {
            let amount = u64::from(index + 1) * 1_000;
            let tag = index as u8 + 1;
            let asset = AssetId::from_byte_array([tag; 32]);
            let mut input = UTXO::explicit(
                OutPoint::new(Txid::from_byte_array([1; 32]), index),
                Script::from(vec![0x51, tag]),
                amount,
                asset,
            );
            input.txout.asset = Asset::new_confidential(&secp, asset, asset_blinder);
            input.txout.value = Value::new_confidential_from_assetid(
                &secp,
                amount,
                asset,
                value_blinder,
                asset_blinder,
            );
            input.secrets = Some(TxOutSecrets::new(
                asset,
                asset_blinder,
                amount,
                value_blinder,
            ));
            let output = PartialOutput::new(signer.get_address().script_pubkey(), amount, asset)
                .with_blinding_key(signer.get_blinding_public_key());

            transaction.add_input(PartialInput::new(input), None);
            transaction.add_output(output);
        }

        // Keep the transaction balanced while giving each position a different output.
        transaction.outputs_mut().reverse();
        transaction
    }
}
