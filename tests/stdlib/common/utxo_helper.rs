use simplex::program::Program;
use simplex::simplicityhl::elements::OutPoint;
use simplex::simplicityhl::elements::Txid;
use simplex::simplicityhl::elements::pset::serialize::Serialize;
use simplex::transaction::UTXO;

use crate::common::core::{send_blinded, send_explicit};
pub const DEFAULT_SEND_AMOUNT: u64 = 50;

pub struct ConfidentialAssetId {
    pub parity_bit: u8,
    pub asset_id: [u8; 32],
}
pub struct ConfidentialAmount {
    pub parity_bit: u8,
    pub amount: [u8; 32],
}

pub type ConfidentialAsset = (ConfidentialAssetId, ConfidentialAmount);

pub fn create_utxo_for_inputs(
    context: &simplex::TestContext,
    index: u32,
    is_explicit_input: bool,
    is_input: bool,
    expected_amount: u64,
    program: impl AsRef<Program>,
) -> anyhow::Result<(Option<ConfidentialAsset>, Vec<Txid>)> {
    let signer = context.get_default_signer();
    let pubkey_script = &signer.get_address().script_pubkey();
    let program_script = &program.as_ref().get_script_pubkey(context.get_network());

    let mut txids = Vec::new();

    for i in 0..=index {
        let script = match i == 0 {
            true => program_script,
            false => pubkey_script,
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

        if index == i && !is_explicit_input {
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

    if !is_explicit_input {
        let script = match index == 0 {
            true => program_script,
            false => pubkey_script,
        };

        let utxos = context
            .get_default_provider()
            .fetch_scripthash_utxos(script)?;

        let conf_utxo = search_utxo_by_txid(txids.last().unwrap(), &utxos)?;

        Ok((Some(get_asset_id_amount_from_conf_utxo(conf_utxo)?), txids))
    } else {
        Ok((None, txids))
    }
}

fn get_asset_id_amount_from_conf_utxo(conf_utxo: UTXO) -> anyhow::Result<ConfidentialAsset> {
    let confidential_asset = conf_utxo.txout.asset.serialize(); // 0x0a | 0x0b
    let confidential_amount = conf_utxo.txout.value.serialize(); // 0x08 | 0x09

    // 0x0a = 00001010 & 1 = 0
    // 0x0b = 00001011 & 1 = 1
    // 0x08 = 00001000 & 1 = 0
    // 0x09 = 00001001 & 1 = 1
    let asset_parity_bit = confidential_asset[0] & 1;
    let amount_parity_bit = confidential_amount[0] & 1;

    let asset_id: [u8; 32] = confidential_asset[1..33].try_into()?;
    let amount: [u8; 32] = confidential_amount[1..33].try_into()?;

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
