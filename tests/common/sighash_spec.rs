//! An implementation of `docs/sighash_elements.md`, written from the document rather than from
//! the contract, so that tests can check one against the other.

// Each `tests/*.rs` is a separate crate that mounts this module but uses only
// part of it, so per-crate dead-code analysis would warn about the rest.
#![allow(dead_code)]

use simplex::simplicityhl::elements::confidential::{Asset, Value};
use simplex::simplicityhl::elements::encode::serialize;
use simplex::simplicityhl::elements::hashes::{Hash, HashEngine, sha256};
use simplex::simplicityhl::elements::secp256k1_zkp::{RangeProof, ZERO_TWEAK};
use simplex::simplicityhl::elements::taproot::ControlBlock;
use simplex::simplicityhl::elements::{BlockHash, Transaction, TxIn, TxOut};

pub const ALL: u8 = 0x01;
pub const NONE: u8 = 0x02;
pub const SINGLE: u8 = 0x03;
pub const ALL_ANYONECANPAY: u8 = 0x81;
pub const NONE_ANYONECANPAY: u8 = 0x82;
pub const SINGLE_ANYONECANPAY: u8 = 0x83;

pub const MODES: [u8; 6] = [
    ALL,
    NONE,
    SINGLE,
    ALL_ANYONECANPAY,
    NONE_ANYONECANPAY,
    SINGLE_ANYONECANPAY,
];

/// Everything the signed message depends on.
pub struct SighashContext<'a> {
    pub tx: &'a Transaction,
    /// The outputs spent by `tx`'s inputs, in input order.
    pub utxos: &'a [TxOut],
    /// The position of the input being signed.
    pub index: u32,
    pub control_block: &'a ControlBlock,
    pub cmr: [u8; 32],
    pub genesis: BlockHash,
}

/// The signed message for `mode`, or `None` if the spend is invalid in that mode.
pub fn sighash(ctx: &SighashContext, mode: u8) -> Option<[u8; 32]> {
    if mode == ALL {
        return Some(sig_all_hash(ctx));
    }

    let mode_hash = mode_hash(ctx, mode)?;
    let mut outer = Vec::with_capacity(132);
    outer.extend(sha(b"simplicityhl-std\x1fSighash"));
    outer.extend(genesis_bytes(ctx));
    outer.extend(mode_hash);
    outer.extend(tap_env(ctx));
    if mode & 0x80 == 0 {
        outer.extend(ctx.index.to_be_bytes());
    }
    Some(sha(&outer))
}

/// The tagged hash for one of the five custom modes, or `None` if the spend is invalid in that
/// mode.
pub fn mode_hash(ctx: &SighashContext, mode: u8) -> Option<[u8; 32]> {
    let (name, fields): (&str, Vec<[u8; 32]>) = match mode {
        NONE => ("none", all_inputs(ctx).to_vec()),
        SINGLE => {
            let mut fields = all_inputs(ctx).to_vec();
            fields.push(output_hash(ctx.tx.output.get(ctx.index as usize)?));
            fields.push(output_surjection_proof(&ctx.tx.output[ctx.index as usize]));
            ("single", fields)
        }
        ALL_ANYONECANPAY => {
            let mut fields = current_input(ctx).to_vec();
            fields.push(outputs_hash(ctx.tx));
            ("all_anyonecanpay", fields)
        }
        NONE_ANYONECANPAY => ("none_anyonecanpay", current_input(ctx).to_vec()),
        SINGLE_ANYONECANPAY => {
            let mut fields = current_input(ctx).to_vec();
            fields.push(output_hash(ctx.tx.output.get(ctx.index as usize)?));
            ("single_anyonecanpay", fields)
        }
        _ => return None,
    };

    let tag = sha(format!("simplicityhl-std\x1fSighash\x1f{name}").as_bytes());
    let mut preimage = Vec::new();
    preimage.extend(tag);
    preimage.extend(tag);
    preimage.extend(ctx.tx.version.to_be_bytes());
    preimage.extend(ctx.tx.lock_time.to_consensus_u32().to_be_bytes());
    for field in fields {
        preimage.extend(field);
    }
    Some(sha(&preimage))
}

/// Simplicity's `sig_all_hash`, as restated in the spec's ALL section.
pub fn sig_all_hash(ctx: &SighashContext) -> [u8; 32] {
    let tx = ctx.tx;
    let [inputs, utxos, issuances] = all_inputs(ctx);
    let mut tx_preimage = Vec::new();
    tx_preimage.extend(tx.version.to_be_bytes());
    tx_preimage.extend(tx.lock_time.to_consensus_u32().to_be_bytes());
    tx_preimage.extend(inputs);
    tx_preimage.extend(outputs_hash(tx));
    tx_preimage.extend(issuances);
    tx_preimage.extend(output_surjection_proofs_hash(tx));
    tx_preimage.extend(utxos);

    let mut preimage = Vec::new();
    preimage.extend(genesis_bytes(ctx));
    preimage.extend(genesis_bytes(ctx));
    preimage.extend(sha(&tx_preimage));
    preimage.extend(tap_env(ctx));
    preimage.extend(ctx.index.to_be_bytes());
    sha(&preimage)
}

pub fn tap_env(ctx: &SighashContext) -> [u8; 32] {
    let cb = ctx.control_block.serialize();
    let leaf_tag = sha(b"TapLeaf/elements");
    let mut leaf = Vec::new();
    leaf.extend(leaf_tag);
    leaf.extend(leaf_tag);
    leaf.push(cb[0] & 0xfe);
    leaf.push(0x20);
    leaf.extend(ctx.cmr);

    let mut preimage = Vec::new();
    preimage.extend(sha(&leaf));
    preimage.extend(sha(&cb[33..]));
    preimage.extend(&cb[1..33]);
    sha(&preimage)
}

/// `inputs_hash`, `input_utxos_hash` and `issuances_hash`.
fn all_inputs(ctx: &SighashContext) -> [[u8; 32]; 3] {
    let inputs = &ctx.tx.input;
    let concat = |f: &dyn Fn(usize, &TxIn) -> Vec<u8>| {
        sha(&inputs
            .iter()
            .enumerate()
            .flat_map(|(i, input)| f(i, input))
            .collect::<Vec<_>>())
    };

    let outpoints = concat(&|_, input| {
        let mut b = pegin(input);
        b.extend(input.previous_output.txid.to_byte_array());
        b.extend(input.previous_output.vout.to_be_bytes());
        b
    });
    let sequences = concat(&|_, input| input.sequence.to_consensus_u32().to_be_bytes().to_vec());
    let annexes = concat(&|_, input| annex(input));
    let utxo_amounts = concat(&|i, _| {
        let mut b = encode_asset(&ctx.utxos[i].asset);
        b.extend(encode_amount(&ctx.utxos[i].value));
        b
    });
    let utxo_scripts = concat(&|i, _| sha(ctx.utxos[i].script_pubkey.as_bytes()).to_vec());
    let issuance_amounts = concat(&|_, input| match issuance(input) {
        None => vec![0x00, 0x00],
        Some(issuance) => issuance.asset,
    });
    let issuance_tokens = concat(&|_, input| match issuance(input) {
        None => vec![0x00, 0x00],
        Some(issuance) => issuance.tokens,
    });
    let issuance_range_proofs = concat(&|_, input| {
        let (asset_proof, token_proof) = issuance_range_proofs(input);
        [asset_proof, token_proof].concat()
    });
    let issuance_entropies = concat(&|_, input| match issuance(input) {
        None => vec![0x00],
        Some(issuance) => issuance.entropy,
    });

    [
        sha(&[outpoints, sequences, annexes].concat()),
        sha(&[utxo_amounts, utxo_scripts].concat()),
        sha(&[
            issuance_amounts,
            issuance_tokens,
            issuance_range_proofs,
            issuance_entropies,
        ]
        .concat()),
    ]
}

/// `input_hash(index)`, `input_utxo_hash(index)` and `issuance_hash(index)`.
fn current_input(ctx: &SighashContext) -> [[u8; 32]; 3] {
    let i = ctx.index as usize;
    let input = &ctx.tx.input[i];
    let utxo = &ctx.utxos[i];

    let mut input_preimage = pegin(input);
    input_preimage.extend(input.previous_output.txid.to_byte_array());
    input_preimage.extend(input.previous_output.vout.to_be_bytes());
    input_preimage.extend(input.sequence.to_consensus_u32().to_be_bytes());
    input_preimage.extend(annex(input));

    let mut utxo_preimage = encode_asset(&utxo.asset);
    utxo_preimage.extend(encode_amount(&utxo.value));
    utxo_preimage.extend(sha(utxo.script_pubkey.as_bytes()));

    let (asset_proof, token_proof) = issuance_range_proofs(input);
    let issuance_preimage = match issuance(input) {
        None => [
            vec![0x00, 0x00, 0x00, 0x00],
            asset_proof.to_vec(),
            token_proof.to_vec(),
            vec![0x00],
        ]
        .concat(),
        Some(issuance) => [
            issuance.asset,
            issuance.tokens,
            asset_proof.to_vec(),
            token_proof.to_vec(),
            issuance.entropy,
        ]
        .concat(),
    };

    [
        sha(&input_preimage),
        sha(&utxo_preimage),
        sha(&issuance_preimage),
    ]
}

fn output_hash(output: &TxOut) -> [u8; 32] {
    let mut preimage = encode_asset(&output.asset);
    preimage.extend(encode_amount(&output.value));
    preimage.extend(serialize(&output.nonce));
    preimage.extend(sha(output.script_pubkey.as_bytes()));
    preimage.extend(range_proof(output));
    sha(&preimage)
}

fn outputs_hash(tx: &Transaction) -> [u8; 32] {
    let concat =
        |f: &dyn Fn(&TxOut) -> Vec<u8>| sha(&tx.output.iter().flat_map(f).collect::<Vec<_>>());
    let amounts = concat(&|o| [encode_asset(&o.asset), encode_amount(&o.value)].concat());
    let nonces = concat(&|o| serialize(&o.nonce));
    let scripts = concat(&|o| sha(o.script_pubkey.as_bytes()).to_vec());
    let range_proofs = concat(&|o| range_proof(o).to_vec());
    sha(&[amounts, nonces, scripts, range_proofs].concat())
}

fn output_surjection_proofs_hash(tx: &Transaction) -> [u8; 32] {
    sha(&tx
        .output
        .iter()
        .flat_map(output_surjection_proof)
        .collect::<Vec<_>>())
}

fn output_surjection_proof(output: &TxOut) -> [u8; 32] {
    match (&output.asset, &output.witness.surjection_proof) {
        (Asset::Confidential(_), Some(proof)) => sha(&proof.serialize()),
        _ => sha(&[]),
    }
}

/// `H(range proof)`, counting the proof only for a confidential amount.
fn range_proof(output: &TxOut) -> [u8; 32] {
    confidential_proof(&output.value, &output.witness.rangeproof)
}

fn pegin(input: &TxIn) -> Vec<u8> {
    if input.is_pegin {
        [&[0x01][..], &input.witness.pegin_witness[2]].concat()
    } else {
        vec![0x00]
    }
}

/// The spec's annex rule: at least two witness items, the last starting with `0x50`.
fn annex(input: &TxIn) -> Vec<u8> {
    let stack = &input.witness.script_witness;
    match stack.last() {
        Some(last) if stack.len() >= 2 && last.first() == Some(&0x50) => {
            [&[0x01][..], &sha(&last[1..])].concat()
        }
        _ => vec![0x00],
    }
}

struct Issuance {
    /// `issued_asset`
    asset: Vec<u8>,
    /// `issued_tokens`
    tokens: Vec<u8>,
    /// `entropy_data`
    entropy: Vec<u8>,
}

fn issuance(input: &TxIn) -> Option<Issuance> {
    let issuance = &input.asset_issuance;
    if issuance.amount.is_null() && issuance.inflation_keys.is_null() {
        return None;
    }

    let (asset_id, token_id) = input.issuance_ids();
    let explicit = |id: [u8; 32]| [&[0x01][..], &id].concat();
    let is_new = issuance.asset_blinding_nonce == ZERO_TWEAK;
    let token_amount = if is_new {
        issuance.inflation_keys
    } else {
        Value::Explicit(0)
    };

    Some(Issuance {
        asset: [
            explicit(asset_id.into_inner().to_byte_array()),
            encode_amount(&issuance.amount),
        ]
        .concat(),
        tokens: [
            explicit(token_id.into_inner().to_byte_array()),
            encode_amount(&token_amount),
        ]
        .concat(),
        entropy: if is_new {
            [&[0x01][..], &[0u8; 32], &issuance.asset_entropy].concat()
        } else {
            [
                &[0x01][..],
                issuance.asset_blinding_nonce.as_ref(),
                &issuance.asset_entropy,
            ]
            .concat()
        },
    })
}

/// `H(asset range proof)` and `H(token range proof)` for the input's issuance.
fn issuance_range_proofs(input: &TxIn) -> ([u8; 32], [u8; 32]) {
    let issuance = &input.asset_issuance;
    let has_issuance = !(issuance.amount.is_null() && issuance.inflation_keys.is_null());
    let is_new = issuance.asset_blinding_nonce == ZERO_TWEAK;
    if !has_issuance {
        return (sha(&[]), sha(&[]));
    }
    let asset = confidential_proof(&issuance.amount, &input.witness.amount_rangeproof);
    let token = if is_new {
        confidential_proof(
            &issuance.inflation_keys,
            &input.witness.inflation_keys_rangeproof,
        )
    } else {
        sha(&[])
    };
    (asset, token)
}

/// `H(proof)` if `value` is confidential, otherwise `H(empty)`.
fn confidential_proof(value: &Value, proof: &Option<Box<RangeProof>>) -> [u8; 32] {
    match (value, proof) {
        (Value::Confidential(_), Some(proof)) => sha(&proof.serialize()),
        _ => sha(&[]),
    }
}

fn encode_asset(asset: &Asset) -> Vec<u8> {
    serialize(asset)
}

/// A null amount is encoded as explicit 0.
fn encode_amount(value: &Value) -> Vec<u8> {
    match value {
        Value::Null => serialize(&Value::Explicit(0)),
        _ => serialize(value),
    }
}

fn genesis_bytes(ctx: &SighashContext) -> [u8; 32] {
    ctx.genesis.to_byte_array()
}

fn sha(data: &[u8]) -> [u8; 32] {
    let mut engine = sha256::Hash::engine();
    engine.input(data);
    sha256::Hash::from_engine(engine).to_byte_array()
}
