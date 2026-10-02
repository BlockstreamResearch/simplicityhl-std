//! Cost comparison of alternative implementations, answering
//! <https://github.com/BlockstreamResearch/simplicityhl-std/pull/44#issuecomment-5296980474>:
//!
//! - `vh`: is `v*H` cheaper by double-and-add, or by a lookup table when the
//!   asset is known, than by the jets the library uses?
//! - `opening`: is an exact value proof cheaper than blinding factors for
//!   opening a confidential `(asset, amount)` pair?
//!
//! Each program is compiled, executed and pruned against a dummy Elements
//! environment, as consensus would run it, and its cost is read from the
//! pruned program. A candidate is measured only after it produced the right
//! result on every test vector, so a cheap but wrong variant cannot win.
//!
//! Run with `cargo test --test bench -- --nocapture` to see the tables.

#[allow(dead_code)]
#[path = "../stdlib/ct/helpers.rs"]
mod helpers;

mod opening;
mod vh;

use std::ops::Sub;
use std::sync::{Arc, OnceLock};

use secp256k1_zkp::{All, PublicKey, Scalar, Secp256k1};

use simplex::program::WitnessTrait;
use simplex::simplicityhl::ast::ElementsJetHinter;
use simplex::simplicityhl::elements::hashes::Hash;
use simplex::simplicityhl::elements::{self, confidential, taproot::ControlBlock};
use simplex::simplicityhl::simplicity::Cmr;
use simplex::simplicityhl::simplicity::jet::elements::{ElementsEnv, ElementsUtxo};
use simplex::simplicityhl::{Arguments, CompiledProgram, UnstableFeatures, WitnessValues};

use helpers::{Ge, secp};

/// One context for the many multiplications of a lookup table.
fn context() -> &'static Secp256k1<All> {
    static CONTEXT: OnceLock<Secp256k1<All>> = OnceLock::new();
    CONTEXT.get_or_init(secp)
}

/// A minimal transaction environment; the bench programs do not read the transaction.
pub fn env() -> ElementsEnv<Arc<elements::Transaction>> {
    let ctrl_blk: [u8; 33] = [
        0xc0, 0xeb, 0x04, 0xb6, 0x8e, 0x9a, 0x26, 0xd1, 0x16, 0x04, 0x6c, 0x76, 0xe8, 0xff, 0x47,
        0x33, 0x2f, 0xb7, 0x1d, 0xda, 0x90, 0xff, 0x4b, 0xef, 0x53, 0x70, 0xf2, 0x52, 0x26, 0xd3,
        0xbc, 0x09, 0xfc,
    ];

    ElementsEnv::new(
        Arc::new(elements::Transaction {
            version: 2,
            lock_time: elements::LockTime::ZERO,
            input: vec![elements::TxIn {
                previous_output: elements::OutPoint::default(),
                is_pegin: false,
                script_sig: elements::Script::new(),
                sequence: elements::Sequence::MAX,
                asset_issuance: elements::AssetIssuance::default(),
                witness: elements::TxInWitness::default(),
            }],
            output: Vec::default(),
        }),
        vec![ElementsUtxo {
            script_pubkey: elements::Script::new(),
            asset: confidential::Asset::Null,
            value: confidential::Value::Null,
        }],
        0,
        Cmr::from_byte_array([0; 32]),
        ControlBlock::from_slice(&ctrl_blk).unwrap(),
        None,
        elements::BlockHash::from_byte_array([0; 32]),
    )
}

/// What a run costs once pruned: milliweight units and encoded bytes.
#[derive(Clone, Copy)]
pub struct Cost {
    pub milliweight: u64,
    pub bytes: usize,
}

impl Cost {
    /// The weight the input pays for: its budget is 1000 mWU per byte of its
    /// witness stack, which holds the program, so a program is paid for by its
    /// size or, if its cost exceeds that budget, by annex padding.
    pub fn weight(&self) -> u64 {
        (self.bytes as u64).max(self.milliweight.div_ceil(1000))
    }
}

impl Sub for Cost {
    type Output = Cost;

    fn sub(self, base: Cost) -> Cost {
        Cost {
            milliweight: self.milliweight - base.milliweight,
            bytes: self.bytes - base.bytes,
        }
    }
}

pub fn compile(source: &str) -> Result<CompiledProgram, String> {
    CompiledProgram::new_with_unstable(
        source,
        &UnstableFeatures::all(),
        Arguments::default(),
        false,
        Box::new(ElementsJetHinter),
    )
}

/// Executes and prunes `program` with `witness`. Fails if the program fails.
pub fn execute(program: &CompiledProgram, witness: WitnessValues) -> Result<Cost, String> {
    let satisfied = program.satisfy_with_env(witness, Some(&env()))?;
    let redeem = satisfied.redeem();
    let (program_bytes, witness_bytes) = redeem.to_vec_with_witness();

    Ok(Cost {
        milliweight: redeem.bounds().cost.to_string().parse().unwrap(),
        bytes: program_bytes.len() + witness_bytes.len(),
    })
}

/// Compiles `source`, then executes and prunes it with `witness`.
pub fn run(source: &str, witness: &impl WitnessTrait) -> Result<Cost, String> {
    execute(&compile(source)?, witness.build_witness())
}

/// `jet::sig_all_hash()` of the environment `run` executes in.
pub fn sig_all_hash() -> [u8; 32] {
    env().c_tx_env().sighash_all().to_byte_array()
}

/// `k*p`. Panics if the result is the point at infinity.
pub fn mul(p: Ge, k: u64) -> PublicKey {
    let mut uncompressed = [4u8; 65];
    uncompressed[1..33].copy_from_slice(&p.0);
    uncompressed[33..].copy_from_slice(&p.1);

    let mut scalar = [0u8; 32];
    scalar[24..].copy_from_slice(&k.to_be_bytes());

    PublicKey::from_slice(&uncompressed)
        .unwrap()
        .mul_tweak(context(), &Scalar::from_be_bytes(scalar).unwrap())
        .unwrap()
}

/// Affine coordinates of `p`.
pub fn ge(p: PublicKey) -> Ge {
    let uncompressed = p.serialize_uncompressed();
    (
        uncompressed[1..33].try_into().unwrap(),
        uncompressed[33..].try_into().unwrap(),
    )
}

pub fn print_header(title: &str) {
    println!("\n{title}, cost above baseline");
    println!(
        "{:<44} {:>10} {:>10} {:>7} {:>7}",
        "candidate", "min (mWU)", "max (mWU)", "bytes", "WU"
    );
}

/// One row over the costs of several runs; `bytes` and `WU` are their maxima.
pub fn print_row(name: &str, costs: &[Cost]) {
    let min = costs.iter().map(|c| c.milliweight).min().unwrap();
    let max = costs.iter().map(|c| c.milliweight).max().unwrap();
    let bytes = costs.iter().map(|c| c.bytes).max().unwrap();
    let weight = costs.iter().map(Cost::weight).max().unwrap();

    println!("{name:<44} {min:>10} {max:>10} {bytes:>7} {weight:>7}");
}
