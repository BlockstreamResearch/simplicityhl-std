use primitive_types::U256;

use simplex::program::Program;
use simplex::simplicityhl::elements::Script;
use simplex::simplicityhl::elements::hex::ToHex;
use simplex::simplicityhl::elements::pset::serialize::Serialize;
use simplex::transaction::UTXO;
use simplex::{simplicityhl::elements::OutPoint, transaction::TxReceipt};

use crate::common::core::{send_explicit, send_with_blinding_return_txid};

const DEFAULT_SEND_AMOUNT: u64 = 50;

pub struct ConfidentialAssetId {
    pub parity_bit: u64,
    pub asset_id: [u8; 32],
}
pub struct ConfidentialAmount {
    pub parity_bit: u64,
    pub amount: [u8; 32],
}

pub fn from_hex_to_u256_array(str: &str) -> anyhow::Result<[u8; 32]> {
    assert!(str.len() <= 64);

    let result = U256::from_str_radix(str, 16)?;

    Ok(result.to_big_endian())
}

pub fn create_utxo_for_inputs(
    context: &simplex::TestContext,
    index: u32,
    is_explicit_input: bool,
    is_input: bool,
    expected_amount: u64,
    program: impl AsRef<Program>,
) -> anyhow::Result<Option<(ConfidentialAssetId, ConfidentialAmount)>> {
    let signer = context.get_default_signer();
    let pubkey_script = &signer.get_address().script_pubkey();
    let program_script = &program.as_ref().get_script_pubkey(context.get_network());

    let mut receipt_result: Option<TxReceipt> = None;

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
            receipt_result = Some(send_with_blinding_return_txid(
                signer,
                script,
                amount_to_send,
                context.get_network().policy_asset(),
            )?);
        } else {
            send_explicit(signer, script, amount_to_send)?;
        }
    }

    if !is_explicit_input {
        let receipt = match receipt_result {
            Some(value) => value,
            None => panic!("Confidential utxo was not created"),
        };

        let script = match index == 0 {
            true => program_script,
            false => pubkey_script,
        };

        let conf_utxo = get_utxo_by_receipt(context, receipt, script)?;

        Ok(Some(get_asset_id_amount_from_conf_utxo(conf_utxo)?))
    } else {
        Ok(None)
    }
}

fn get_asset_id_amount_from_conf_utxo(
    conf_utxo: UTXO,
) -> anyhow::Result<(ConfidentialAssetId, ConfidentialAmount)> {
    let confidential_asset: String = conf_utxo.txout.asset.serialize().to_hex();
    let asset_bytes_arr = &confidential_asset[0..2];

    let asset_parity_bit = match asset_bytes_arr {
        "0a" => 0_u64,
        "0b" => 1_u64,
        _ => panic!("Unknown parity bit, should be 0a or 0b"),
    };
    let asset_id = from_hex_to_u256_array(&confidential_asset[2..66])?;

    let confidential_amount: String = conf_utxo.txout.value.serialize().to_hex();
    let amount_bytes_arr = &confidential_amount[0..2];
    let amount_parity_bit: i32 = amount_bytes_arr.parse().unwrap();

    let amount_parity_bit = match amount_parity_bit {
        8 => 0_u64,
        9 => 1_u64,
        _ => panic!("Unknown parity bit, should be 8 or 9"),
    };
    let amount = from_hex_to_u256_array(&confidential_amount[2..66])?;

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

fn get_utxo_by_receipt(
    context: &simplex::TestContext,
    receipt: TxReceipt,
    script: &Script,
) -> anyhow::Result<UTXO> {
    let outpoint = OutPoint {
        txid: receipt.txid(),
        vout: 0,
    };

    let utxos = context
        .get_default_provider()
        .fetch_scripthash_utxos(script)?;

    utxos
        .iter()
        .find(|utxo| utxo.outpoint == outpoint)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing confidential utxo"))
}
