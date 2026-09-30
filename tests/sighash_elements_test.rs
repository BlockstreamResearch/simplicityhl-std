//! Tests for `sighash_elements` that check what each mode commits to.
//!
//! Each test signs a base transaction in a mode, changes one thing about the transaction, and
//! checks whether the signature still verifies. The signature must survive exactly the changes
//! that the mode leaves out of its hash: ANYONECANPAY lets other inputs change, SINGLE lets
//! other outputs change, NONE lets any output change, and nothing lets the signed input change.
//!
//! The contract is run locally against transactions built here, so the tests can make changes
//! a wallet never would (issuances, surjection proofs, other chains). The message to sign is
//! taken from the contract itself: it runs once with a dummy signature while a tracker records
//! what `bip_0340_verify` was asked to check.

mod common;

use std::str::FromStr;
use std::sync::Arc;

use simplex::program::{ArgumentsTrait, ProgramTrait, WitnessTrait};
use simplex::provider::SimplicityNetwork;
use simplex::simplicityhl::ast::ElementsJetHinter;
use simplex::simplicityhl::elements::confidential::{Asset, Nonce, Value};
use simplex::simplicityhl::elements::pset::PartiallySignedTransaction;
use simplex::simplicityhl::elements::secp256k1_zkp::rand::thread_rng;
use simplex::simplicityhl::elements::secp256k1_zkp::{
    Generator, Keypair, Message, PedersenCommitment, RangeProof, Secp256k1, SecretKey,
    SurjectionProof, Tag, Tweak, ZERO_TWEAK,
};
use simplex::simplicityhl::elements::taproot::ControlBlock;
use simplex::simplicityhl::elements::{
    AssetIssuance, LockTime, OutPoint, Script, Sequence, Transaction, TxIn, TxOut, TxOutWitness,
    Txid,
};
use simplex::simplicityhl::simplicity::BitMachine;
use simplex::simplicityhl::simplicity::Cmr;
use simplex::simplicityhl::simplicity::bit_machine::ExecutionError;
use simplex::simplicityhl::simplicity::jet::elements::{ElementsEnv, ElementsUtxo};
use simplex::simplicityhl::tracker::DefaultTracker;
use simplex::simplicityhl::{CompiledProgram, UnstableFeatures};

use common::core::{Expect, run_signed};

use simplicityhl_std::artifacts::sighash_elements_test::SighashElementsTestProgram;
use simplicityhl_std::artifacts::sighash_elements_test::derived_sighash_elements_test::{
    SighashElementsTestArguments, SighashElementsTestWitness,
};

// BIP-341 `hash_type` bytes.
const ALL: u8 = 0x01;
const NONE: u8 = 0x02;
const SINGLE: u8 = 0x03;
const ALL_ANYONECANPAY: u8 = 0x81;
const NONE_ANYONECANPAY: u8 = 0x82;
const SINGLE_ANYONECANPAY: u8 = 0x83;

const MODES: [u8; 6] = [
    ALL,
    NONE,
    SINGLE,
    ALL_ANYONECANPAY,
    NONE_ANYONECANPAY,
    SINGLE_ANYONECANPAY,
];

const ANYONECANPAY: [u8; 3] = [ALL_ANYONECANPAY, NONE_ANYONECANPAY, SINGLE_ANYONECANPAY];
/// The modes that leave out every output except, for SINGLE, the paired one.
const NOT_OTHER_OUTPUTS: [u8; 4] = [NONE, SINGLE, NONE_ANYONECANPAY, SINGLE_ANYONECANPAY];
const NO_OUTPUTS: [u8; 2] = [NONE, NONE_ANYONECANPAY];

/// The contract's input sits at this index in the base transaction.
const PROGRAM_INPUT: usize = 0;
/// Another party's input.
const OTHER_INPUT: usize = 1;
/// The output SINGLE pairs with the contract's input.
const PAIRED_OUTPUT: usize = 0;
/// An output SINGLE does not sign.
const OTHER_OUTPUT: usize = 1;

fn mode_name(mode: u8) -> &'static str {
    match mode {
        ALL => "ALL",
        NONE => "NONE",
        SINGLE => "SINGLE",
        ALL_ANYONECANPAY => "ALL|ANYONECANPAY",
        NONE_ANYONECANPAY => "NONE|ANYONECANPAY",
        SINGLE_ANYONECANPAY => "SINGLE|ANYONECANPAY",
        _ => "invalid",
    }
}

/// How a run of the contract ended.
#[derive(Debug, PartialEq)]
enum Outcome {
    Verified,
    /// `bip_0340_verify` rejected the signature.
    Rejected,
    /// The contract reached a `panic!()`.
    Panicked,
}

/// A transaction together with the UTXOs its inputs spend, which the sighash also covers.
#[derive(Clone)]
struct Spend {
    tx: Transaction,
    utxos: Vec<TxOut>,
    network: SimplicityNetwork,
}

/// The contract and a key to sign for it with.
struct Harness {
    program: SighashElementsTestProgram,
    compiled: CompiledProgram,
    keypair: Keypair,
    network: SimplicityNetwork,
}

impl Harness {
    fn new() -> Self {
        Self::with_secret_key([7u8; 32])
    }

    fn with_secret_key(secret: [u8; 32]) -> Self {
        let arguments = SighashElementsTestArguments {};
        let compiled = CompiledProgram::new_with_unstable(
            SighashElementsTestProgram::SOURCE,
            &UnstableFeatures::all(),
            arguments.build_arguments(),
            false,
            Box::new(ElementsJetHinter),
        )
        .expect("test contract compiles");

        Self {
            program: SighashElementsTestProgram::new(&arguments),
            compiled,
            keypair: Keypair::from_seckey_slice(&Secp256k1::new(), &secret).unwrap(),
            network: SimplicityNetwork::default_regtest(),
        }
    }

    fn pubkey(&self) -> [u8; 32] {
        self.keypair.x_only_public_key().0.serialize()
    }

    /// Two inputs (the contract's, then another party's) and two outputs.
    fn base_spend(&self) -> Spend {
        Spend {
            tx: Transaction {
                version: 2,
                lock_time: LockTime::ZERO,
                input: vec![input(1), input(2)],
                output: vec![self.output(b"paired", 60), self.output(b"other", 40)],
            },
            utxos: vec![
                self.output_to(self.program.get_script_pubkey(&self.network), 50),
                self.output(b"other party's coin", 50),
            ],
            network: self.network,
        }
    }

    fn output(&self, tag: &[u8], value: u64) -> TxOut {
        self.output_to(Script::new_op_return(tag), value)
    }

    fn output_to(&self, script_pubkey: Script, value: u64) -> TxOut {
        TxOut {
            asset: Asset::Explicit(self.network.policy_asset()),
            value: Value::Explicit(value),
            nonce: Nonce::Null,
            script_pubkey,
            witness: TxOutWitness::default(),
        }
    }

    /// Run the contract for the input at `index`, checking `sig` against `pubkey`. `on_verify`
    /// sees the message that reaches `bip_0340_verify`, if execution gets that far.
    fn execute(
        &self,
        spend: &Spend,
        index: usize,
        mode: u8,
        pubkey: [u8; 32],
        sig: [u8; 64],
        mut on_verify: impl FnMut([u8; 32]),
    ) -> Outcome {
        let witness = SighashElementsTestWitness { mode, pubkey, sig };
        let satisfied = self
            .compiled
            .satisfy(witness.build_witness())
            .expect("witness matches the contract");

        let env = self.env(spend, index);
        let mut tracker =
            DefaultTracker::build(satisfied.debug_symbols(), Box::new(ElementsJetHinter))
                .with_jet_trace_sink(|jet, args, _| {
                    if format!("{jet:?}") == "Bip0340Verify" {
                        on_verify(verified_message(args.expect("arguments decode")));
                    }
                });

        let redeem = satisfied.redeem();
        let result = BitMachine::for_program(redeem)
            .expect("program fits the bit machine")
            .exec_with_tracker(redeem, &env, &mut tracker);

        match result {
            Ok(_) => Outcome::Verified,
            Err(ExecutionError::JetFailed(_)) => Outcome::Rejected,
            Err(ExecutionError::ReachedFailNode(_)) => Outcome::Panicked,
            Err(e) => panic!("unexpected execution error: {e}"),
        }
    }

    /// The environment the contract sees when spending the input at `index`.
    ///
    /// This is built from `spend.tx` directly. Going through a PSET, as the SDK does, would not
    /// give the same transaction back: rust-elements' `PartiallySignedTransaction::from_tx` puts
    /// the issuance and peg-in flags into each input's `vout`, and drops the nonce of outputs
    /// that are not blinded.
    fn env(&self, spend: &Spend, index: usize) -> ElementsEnv<Arc<Transaction>> {
        let utxos = spend
            .utxos
            .iter()
            .map(|utxo| ElementsUtxo {
                script_pubkey: utxo.script_pubkey.clone(),
                asset: utxo.asset,
                value: utxo.value,
            })
            .collect();

        ElementsEnv::new(
            Arc::new(spend.tx.clone()),
            utxos,
            index as u32,
            Cmr::from_byte_array(self.program.get_cmr()),
            self.control_block(spend, index),
            None,
            spend.network.genesis_block_hash(),
        )
    }

    /// The contract's control block, which does not depend on the transaction.
    fn control_block(&self, spend: &Spend, index: usize) -> ControlBlock {
        let mut pst = PartiallySignedTransaction::from_tx(spend.tx.clone());
        for (input, utxo) in pst.inputs_mut().iter_mut().zip(&spend.utxos) {
            input.witness_utxo = Some(utxo.clone());
        }
        self.program
            .as_ref()
            .get_env(&pst, index, &spend.network)
            .expect("the input spends the contract")
            .control_block()
            .clone()
    }

    /// The message the contract checks for `mode` at `index`, or `None` if it panics first.
    fn contract_sighash(&self, spend: &Spend, index: usize, mode: u8) -> Option<[u8; 32]> {
        let mut message = None;
        self.execute(spend, index, mode, self.pubkey(), [0u8; 64], |m| {
            message = Some(m)
        });
        message
    }

    /// The message the contract checks for `mode`, found by running it with a dummy signature.
    fn sighash(&self, spend: &Spend, mode: u8) -> [u8; 32] {
        let mut message = None;
        self.execute(spend, PROGRAM_INPUT, mode, self.pubkey(), [0u8; 64], |m| {
            message = Some(m)
        });
        message.unwrap_or_else(|| panic!("{} produced no sighash", mode_name(mode)))
    }

    fn sign(&self, spend: &Spend, mode: u8) -> [u8; 64] {
        let message = Message::from_digest(self.sighash(spend, mode));

        Secp256k1::new()
            .sign_schnorr_no_aux_rand(&message, &self.keypair)
            .serialize()
    }

    fn verify(&self, spend: &Spend, mode: u8, sig: [u8; 64]) -> Outcome {
        self.execute(spend, PROGRAM_INPUT, mode, self.pubkey(), sig, |_| {})
    }

    /// Sign the base transaction in every mode, apply `change`, and check that exactly the
    /// modes in `survives` still verify.
    fn check_change(&self, change: impl Fn(&mut Spend), survives: &[u8]) {
        self.check_change_from(self.base_spend(), change, PROGRAM_INPUT, survives);
    }

    /// As `check_change`, starting from `base` instead of the base transaction, and verifying
    /// the contract's input at `index` in the changed transaction.
    fn check_change_from(
        &self,
        base: Spend,
        change: impl Fn(&mut Spend),
        index: usize,
        survives: &[u8],
    ) {
        let mut changed = base.clone();
        change(&mut changed);

        let mismatches: Vec<String> = MODES
            .iter()
            .filter_map(|&mode| {
                let sig = self.sign(&base, mode);
                let outcome = self.execute(&changed, index, mode, self.pubkey(), sig, |_| {});
                let expected = if survives.contains(&mode) {
                    Outcome::Verified
                } else {
                    Outcome::Rejected
                };

                (outcome != expected).then(|| {
                    format!(
                        "{}: expected {expected:?}, got {outcome:?}",
                        mode_name(mode)
                    )
                })
            })
            .collect();

        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }
}

fn input(n: u8) -> TxIn {
    TxIn {
        previous_output: OutPoint::new(Txid::from_str(&format!("{n:064x}")).unwrap(), 0),
        is_pegin: false,
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        asset_issuance: AssetIssuance::default(),
        witness: Default::default(),
    }
}

/// `bip_0340_verify` takes `((pubkey, message), signature)`; pull out the message.
fn verified_message(args: &[simplex::simplicityhl::Value]) -> [u8; 32] {
    // The pair displays as `(0x<pubkey>, 0x<message>)`.
    let pair = args[0].to_string();
    let hex = pair
        .trim_end_matches(')')
        .rsplit("0x")
        .next()
        .expect("message is hex");

    let mut message = [0u8; 32];
    for (i, byte) in message.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
    }
    message
}

fn new_issuance() -> AssetIssuance {
    AssetIssuance {
        asset_blinding_nonce: ZERO_TWEAK,
        asset_entropy: [9u8; 32],
        amount: Value::Explicit(1_000),
        inflation_keys: Value::Null,
    }
}

/// A surjection proof that an output of the policy asset, blinded with `blinding`, spends one
/// of `inputs` input assets (the policy asset and `inputs - 1` others).
fn surjection_proof(
    network: &SimplicityNetwork,
    blinding: Tweak,
    inputs: u8,
) -> Box<SurjectionProof> {
    let secp = Secp256k1::new();
    let tag = policy_tag(network);
    let domain: Vec<_> = std::iter::once(tag)
        .chain((1..inputs).map(|i| Tag::from([i; 32])))
        .map(|t| (Generator::new_unblinded(&secp, t), t, ZERO_TWEAK))
        .collect();

    Box::new(SurjectionProof::new(&secp, &mut thread_rng(), tag, blinding, &domain).unwrap())
}

fn policy_tag(network: &SimplicityNetwork) -> Tag {
    Tag::from(network.policy_asset().into_inner().to_byte_array())
}

/// The base transaction with the output at `index` blinded and carrying a surjection proof
/// over its two inputs, plus the proof that output needs once a third input is added.
///
/// Surjection proofs only count for blinded assets: for an explicit asset the environment
/// hashes an empty proof, whatever the output carries.
fn blinded_spend(h: &Harness, index: usize) -> (Spend, Box<SurjectionProof>) {
    let secp = Secp256k1::new();
    let blinding = Tweak::new(&mut thread_rng());
    let mut base = h.base_spend();
    let output = &mut base.tx.output[index];
    output.asset = Asset::Confidential(Generator::new_blinded(
        &secp,
        policy_tag(&h.network),
        blinding,
    ));
    output.witness.surjection_proof = Some(surjection_proof(&h.network, blinding, 2));

    (base, surjection_proof(&h.network, blinding, 3))
}

/// The base transaction with the output at `index` given a confidential amount and a range
/// proof, plus a different valid range proof for the same amount.
///
/// Range proofs only count for confidential amounts: for an explicit amount the environment
/// hashes an empty proof, whatever the output carries.
fn confidential_spend(h: &Harness, index: usize) -> (Spend, Box<RangeProof>) {
    let secp = Secp256k1::new();
    let generator = Generator::new_unblinded(&secp, policy_tag(&h.network));
    let blinding = Tweak::new(&mut thread_rng());
    let amount = 60;
    let commitment = PedersenCommitment::new(&secp, amount, blinding, generator);
    let range_proof = |nonce: u8| {
        let sk = SecretKey::from_slice(&[nonce; 32]).unwrap();
        Box::new(
            RangeProof::new(
                &secp,
                0,
                commitment,
                amount,
                blinding,
                &[],
                &[],
                sk,
                0,
                64,
                generator,
            )
            .unwrap(),
        )
    };

    let mut base = h.base_spend();
    let output = &mut base.tx.output[index];
    output.value = Value::Confidential(commitment);
    output.witness.rangeproof = Some(range_proof(1));

    (base, range_proof(2))
}

/// Puts an annex (a final witness item starting with `0x50`) on `input`.
fn set_annex(input: &mut TxIn, data: u8) {
    input.witness.script_witness = vec![vec![0x01], vec![0x50, data]];
}

/// A nonce, as a blinded output carries for its receiver.
fn nonce(seed: u8) -> Nonce {
    let keypair = Keypair::from_seckey_slice(&Secp256k1::new(), &[seed; 32]).unwrap();
    Nonce::Confidential(keypair.public_key())
}

fn set_value(output: &mut TxOut, value: u64) {
    output.value = Value::Explicit(value);
}

mod mode_selection {
    use super::*;

    #[simplex::test]
    fn signature_is_bound_to_its_mode(_context: simplex::TestContext) -> anyhow::Result<()> {
        // A signature made in one mode must not verify in any other, or a signer who chose a
        // strict mode could have their signature reused under a looser one.
        let h = Harness::new();
        let spend = h.base_spend();

        for signed in MODES {
            let sig = h.sign(&spend, signed);
            for verified in MODES {
                let expected = if signed == verified {
                    Outcome::Verified
                } else {
                    Outcome::Rejected
                };
                assert_eq!(
                    h.verify(&spend, verified, sig),
                    expected,
                    "signed as {}, verified as {}",
                    mode_name(signed),
                    mode_name(verified)
                );
            }
        }
        Ok(())
    }

    #[simplex::test]
    fn invalid_mode_panics(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let spend = h.base_spend();

        for mode in [0x00, 0x04, 0x7f, 0x80, 0x84, 0xff] {
            assert_eq!(
                h.verify(&spend, mode, [0u8; 64]),
                Outcome::Panicked,
                "mode {mode:#04x}"
            );
        }
        Ok(())
    }

    #[simplex::test]
    fn signature_fails_for_another_key(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let other = Harness::with_secret_key([8u8; 32]);
        let spend = h.base_spend();

        for mode in MODES {
            assert_eq!(
                h.verify(&spend, mode, other.sign(&spend, mode)),
                Outcome::Rejected,
                "{}",
                mode_name(mode)
            );
        }
        Ok(())
    }
}

mod what_each_mode_signs {
    use super::*;

    #[simplex::test]
    fn unchanged_transaction(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|_| {}, &MODES);
        Ok(())
    }

    // The signed input itself: every mode commits to it.

    #[simplex::test]
    fn own_sequence(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|s| s.tx.input[PROGRAM_INPUT].sequence = Sequence::ZERO, &[]);
        Ok(())
    }

    #[simplex::test]
    fn own_utxo_value(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|s| set_value(&mut s.utxos[PROGRAM_INPUT], 51), &[]);
        Ok(())
    }

    #[simplex::test]
    fn own_issuance(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| s.tx.input[PROGRAM_INPUT].asset_issuance = new_issuance(),
            &[],
        );
        Ok(())
    }

    #[simplex::test]
    fn own_outpoint(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| s.tx.input[PROGRAM_INPUT].previous_output = input(3).previous_output,
            &[],
        );
        Ok(())
    }

    #[simplex::test]
    fn own_annex(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|s| set_annex(&mut s.tx.input[PROGRAM_INPUT], 1), &[]);
        Ok(())
    }

    #[simplex::test]
    fn other_contract(_context: simplex::TestContext) -> anyhow::Result<()> {
        // The same key guards a second contract, here the same program under another taproot
        // internal key. A signature for one must not spend the other.
        let h = Harness::new();
        let mut other = Harness::new();
        let internal_key = Keypair::from_seckey_slice(&Secp256k1::new(), &[9u8; 32])
            .unwrap()
            .x_only_public_key()
            .0;
        other.program = other.program.with_taproot_pubkey(internal_key);

        for mode in MODES {
            let sig = h.sign(&h.base_spend(), mode);
            assert_eq!(
                other.verify(&other.base_spend(), mode, sig),
                Outcome::Rejected,
                "{}",
                mode_name(mode)
            );
        }
        Ok(())
    }

    // Position of the signed input: only ANYONECANPAY lets it move, since the other modes sign
    // every input anyway.

    #[simplex::test]
    fn input_moves(_context: simplex::TestContext) -> anyhow::Result<()> {
        // Another party's input is inserted in front, so the contract's input ends up at
        // index 1. SINGLE|ANYONECANPAY now pairs with a different output, so it breaks.
        let h = Harness::new();
        let coin = h.output(b"inserted coin", 10);
        h.check_change_from(
            h.base_spend(),
            |s| {
                s.tx.input.insert(0, input(3));
                s.utxos.insert(0, coin.clone());
            },
            PROGRAM_INPUT + 1,
            &[ALL_ANYONECANPAY, NONE_ANYONECANPAY],
        );
        Ok(())
    }

    #[simplex::test]
    fn input_moves_with_its_output(_context: simplex::TestContext) -> anyhow::Result<()> {
        // Another party's input and output are inserted in front, as when two
        // SINGLE|ANYONECANPAY offers are combined. The contract's input and its paired output
        // both end up at index 1.
        let h = Harness::new();
        let coin = h.output(b"inserted coin", 10);
        let payout = h.output(b"inserted payout", 10);
        h.check_change_from(
            h.base_spend(),
            |s| {
                s.tx.input.insert(0, input(3));
                s.utxos.insert(0, coin.clone());
                s.tx.output.insert(0, payout.clone());
            },
            PROGRAM_INPUT + 1,
            &[NONE_ANYONECANPAY, SINGLE_ANYONECANPAY],
        );
        Ok(())
    }

    // Transaction-wide fields: every mode commits to them.

    #[simplex::test]
    fn lock_time(_context: simplex::TestContext) -> anyhow::Result<()> {
        // A height-based lock time with every input final: `jet::tx_lock_time` reports 0 for
        // this, so a mode that hashed it would not notice the change.
        Harness::new().check_change(|s| s.tx.lock_time = LockTime::from_consensus(100), &[]);
        Ok(())
    }

    #[simplex::test]
    fn version(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|s| s.tx.version = 3, &[]);
        Ok(())
    }

    #[simplex::test]
    fn chain(_context: simplex::TestContext) -> anyhow::Result<()> {
        // The same transaction replayed on Liquid instead of regtest.
        Harness::new().check_change(|s| s.network = SimplicityNetwork::Liquid, &[]);
        Ok(())
    }

    // Other inputs: only ANYONECANPAY leaves them out.

    #[simplex::test]
    fn add_input(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let fee_coin = h.output(b"fee payer's coin", 10);
        h.check_change(
            |s| {
                s.tx.input.push(input(3));
                s.utxos.push(fee_coin.clone());
            },
            &ANYONECANPAY,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_outpoint(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|s| s.tx.input[OTHER_INPUT] = input(3), &ANYONECANPAY);
        Ok(())
    }

    #[simplex::test]
    fn other_sequence(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| s.tx.input[OTHER_INPUT].sequence = Sequence::ZERO,
            &ANYONECANPAY,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_utxo_value(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(|s| set_value(&mut s.utxos[OTHER_INPUT], 51), &ANYONECANPAY);
        Ok(())
    }

    #[simplex::test]
    fn other_issuance(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| s.tx.input[OTHER_INPUT].asset_issuance = new_issuance(),
            &ANYONECANPAY,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_annex(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| set_annex(&mut s.tx.input[OTHER_INPUT], 1),
            &ANYONECANPAY,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_utxo_script(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| s.utxos[OTHER_INPUT].script_pubkey = Script::new_op_return(b"another coin"),
            &ANYONECANPAY,
        );
        Ok(())
    }

    // Outputs: SINGLE signs only its paired output, NONE signs none.

    #[simplex::test]
    fn paired_output_value(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| set_value(&mut s.tx.output[PAIRED_OUTPUT], 61),
            &NO_OUTPUTS,
        );
        Ok(())
    }

    #[simplex::test]
    fn paired_output_script(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| s.tx.output[PAIRED_OUTPUT].script_pubkey = Script::new_op_return(b"thief"),
            &NO_OUTPUTS,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_output_value(_context: simplex::TestContext) -> anyhow::Result<()> {
        Harness::new().check_change(
            |s| set_value(&mut s.tx.output[OTHER_OUTPUT], 41),
            &NOT_OTHER_OUTPUTS,
        );
        Ok(())
    }

    #[simplex::test]
    fn add_output(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let extra = h.output(b"extra", 5);
        h.check_change(|s| s.tx.output.push(extra.clone()), &NOT_OTHER_OUTPUTS);
        Ok(())
    }

    // Nonces: only blinded outputs carry one, so these start from a blinded output.

    #[simplex::test]
    fn paired_output_nonce(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let (base, _) = blinded_spend(&h, PAIRED_OUTPUT);
        h.check_change_from(
            base,
            |s| s.tx.output[PAIRED_OUTPUT].nonce = nonce(1),
            PROGRAM_INPUT,
            &NO_OUTPUTS,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_output_nonce(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let (base, _) = blinded_spend(&h, OTHER_OUTPUT);
        h.check_change_from(
            base,
            |s| s.tx.output[OTHER_OUTPUT].nonce = nonce(1),
            PROGRAM_INPUT,
            &NOT_OTHER_OUTPUTS,
        );
        Ok(())
    }

    #[simplex::test]
    fn paired_output_range_proof(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let (base, other_proof) = confidential_spend(&h, PAIRED_OUTPUT);
        h.check_change_from(
            base,
            |s| s.tx.output[PAIRED_OUTPUT].witness.rangeproof = Some(other_proof.clone()),
            PROGRAM_INPUT,
            &NO_OUTPUTS,
        );
        Ok(())
    }

    #[simplex::test]
    fn other_output_range_proof(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let (base, other_proof) = confidential_spend(&h, OTHER_OUTPUT);
        h.check_change_from(
            base,
            |s| s.tx.output[OTHER_OUTPUT].witness.rangeproof = Some(other_proof.clone()),
            PROGRAM_INPUT,
            &NOT_OTHER_OUTPUTS,
        );
        Ok(())
    }

    // Surjection proofs: signed only when every input is, since adding an input means
    // regenerating them.

    #[simplex::test]
    fn paired_output_surjection_proof(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let (base, regenerated) = blinded_spend(&h, PAIRED_OUTPUT);
        h.check_change_from(
            base,
            |s| s.tx.output[PAIRED_OUTPUT].witness.surjection_proof = Some(regenerated.clone()),
            PROGRAM_INPUT,
            &[
                NONE,
                ALL_ANYONECANPAY,
                NONE_ANYONECANPAY,
                SINGLE_ANYONECANPAY,
            ],
        );
        Ok(())
    }

    #[simplex::test]
    fn other_output_surjection_proof(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let (base, regenerated) = blinded_spend(&h, OTHER_OUTPUT);
        h.check_change_from(
            base,
            |s| s.tx.output[OTHER_OUTPUT].witness.surjection_proof = Some(regenerated.clone()),
            PROGRAM_INPUT,
            &[
                NONE,
                SINGLE,
                ALL_ANYONECANPAY,
                NONE_ANYONECANPAY,
                SINGLE_ANYONECANPAY,
            ],
        );
        Ok(())
    }

    #[simplex::test]
    fn single_without_paired_output_panics(_context: simplex::TestContext) -> anyhow::Result<()> {
        // With the contract's input at index 1 and only one output, SINGLE has nothing to pair
        // with. It aborts rather than signing a fixed value the way legacy Bitcoin does.
        let h = Harness::new();
        let mut spend = h.base_spend();
        spend.tx.input.swap(PROGRAM_INPUT, OTHER_INPUT);
        spend.utxos.swap(PROGRAM_INPUT, OTHER_INPUT);
        spend.tx.output.truncate(1);

        for mode in [SINGLE, SINGLE_ANYONECANPAY] {
            assert_eq!(
                h.execute(&spend, OTHER_INPUT, mode, h.pubkey(), [0u8; 64], |_| {}),
                Outcome::Panicked,
                "{}",
                mode_name(mode)
            );
        }
        Ok(())
    }
}

mod on_chain {
    use super::*;

    #[simplex::test]
    fn wallet_all_signature_is_accepted(context: simplex::TestContext) -> anyhow::Result<()> {
        // The simplex wallet signs `sig_all_hash`, so a real spend in ALL mode must be accepted.
        let pubkey = context
            .get_default_signer()
            .get_schnorr_public_key()
            .serialize();

        run_signed(
            &context,
            SighashElementsTestProgram::new(&SighashElementsTestArguments {}),
            SighashElementsTestWitness {
                mode: ALL,
                pubkey,
                sig: [0u8; 64],
            },
            Expect::Ok,
            "SIG",
        )
    }
}

/// Test vectors for `docs/sighash_elements.md`.
///
/// The vectors in `docs/sighash_elements_vectors.json` are produced from the scenarios below by
/// an implementation written from the spec (`tests/common/sighash_spec.rs`). The tests check that
/// the file reproduces from its own contents, that it is current, and that the spec
/// implementation agrees with the contract on every scenario and mode.
///
/// To rewrite the file after a deliberate change, run with `SIGHASH_VECTORS_WRITE=1`.
mod vectors {
    use super::*;

    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use serde_json::{Value as Json, json};
    use simplex::simplicityhl::elements::encode::{deserialize, serialize};
    use simplex::simplicityhl::elements::{BlockHash, hashes::Hash};

    use common::sighash_spec::{self as spec, SighashContext};

    const FILE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/sighash_elements_vectors.json"
    );

    struct Scenario {
        name: &'static str,
        description: &'static str,
        spend: Spend,
        index: usize,
    }

    /// The transactions the vectors are built from. Everything is deterministic.
    fn scenarios(h: &Harness) -> Vec<Scenario> {
        let mut rng = StdRng::seed_from_u64(0x5167_4a54);
        let mut scenarios = Vec::new();

        scenarios.push(Scenario {
            name: "explicit",
            description: "Two inputs and two outputs, all explicit; the contract's input is first.",
            spend: h.base_spend(),
            index: 0,
        });

        let mut spend = h.base_spend();
        spend.tx.input.insert(0, input(3));
        spend.utxos.insert(0, h.output(b"first coin", 30));
        spend.tx.output.push(h.output(b"third", 20));
        spend.tx.version = 3;
        spend.tx.lock_time = LockTime::from_consensus(1_000);
        spend.tx.input[0].sequence = Sequence::from_consensus(5);
        spend.tx.input[1].sequence = Sequence::from_consensus(0xffff_fffe);
        scenarios.push(Scenario {
            name: "second_input",
            description: "Three inputs and three outputs; the contract's input is second. Version \
                          3, a height lock time and non-final sequences.",
            spend,
            index: 1,
        });

        let mut spend = h.base_spend();
        spend.tx.input[OTHER_INPUT].asset_issuance = new_issuance();
        spend.tx.input[PROGRAM_INPUT].asset_issuance = AssetIssuance {
            asset_blinding_nonce: Tweak::new(&mut rng),
            asset_entropy: [0x42; 32],
            amount: Value::Explicit(500),
            inflation_keys: Value::Null,
        };
        scenarios.push(Scenario {
            name: "issuances",
            description: "A new issuance on the other input and a reissuance on the contract's \
                          input.",
            spend,
            index: 0,
        });

        let mut spend = h.base_spend();
        set_annex(&mut spend.tx.input[PROGRAM_INPUT], 0xaa);
        set_annex(&mut spend.tx.input[OTHER_INPUT], 0xbb);
        scenarios.push(Scenario {
            name: "annexes",
            description: "Both inputs carry an annex.",
            spend,
            index: 0,
        });

        let mut spend = h.base_spend();
        spend.tx.input[OTHER_INPUT].witness.script_witness = vec![vec![0x50; 64]];
        scenarios.push(Scenario {
            name: "one_item_witness_starting_0x50",
            description: "The other input's witness is a single item starting with 0x50. It is \
                          not an annex, because an annex needs at least two witness items.",
            spend,
            index: 0,
        });

        let mut spend = h.base_spend();
        blind_output(&mut spend.tx.output[PAIRED_OUTPUT], &h.network, &mut rng);
        scenarios.push(Scenario {
            name: "confidential_output",
            description: "The first output has a confidential asset and amount, a nonce, a range \
                          proof and a surjection proof.",
            spend,
            index: 0,
        });

        let mut spend = h.base_spend();
        spend.tx.input.swap(PROGRAM_INPUT, OTHER_INPUT);
        spend.utxos.swap(PROGRAM_INPUT, OTHER_INPUT);
        spend.tx.output.truncate(1);
        scenarios.push(Scenario {
            name: "no_paired_output",
            description: "The contract's input is second and there is only one output, so SINGLE \
                          and SINGLE|ANYONECANPAY are invalid.",
            spend,
            index: 1,
        });

        scenarios
    }

    /// Give `output` a confidential asset and amount, a nonce, and both proofs.
    fn blind_output(output: &mut TxOut, network: &SimplicityNetwork, rng: &mut StdRng) {
        let secp = Secp256k1::new();
        let tag = policy_tag(network);
        let asset_blinding = Tweak::new(rng);
        let value_blinding = Tweak::new(rng);
        let generator = Generator::new_blinded(&secp, tag, asset_blinding);
        let amount = 60;
        let commitment = PedersenCommitment::new(&secp, amount, value_blinding, generator);
        let domain = [(Generator::new_unblinded(&secp, tag), tag, ZERO_TWEAK)];
        let sk = SecretKey::from_slice(&[3u8; 32]).unwrap();

        output.asset = Asset::Confidential(generator);
        output.value = Value::Confidential(commitment);
        output.nonce = nonce(4);
        output.witness.surjection_proof = Some(Box::new(
            SurjectionProof::new(&secp, rng, tag, asset_blinding, &domain).unwrap(),
        ));
        output.witness.rangeproof = Some(Box::new(
            RangeProof::new(
                &secp,
                0,
                commitment,
                amount,
                value_blinding,
                &[],
                &[],
                sk,
                0,
                64,
                generator,
            )
            .unwrap(),
        ));
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn mode_hex(mode: u8) -> String {
        format!("{mode:#04x}")
    }

    /// The vector for one scenario, computed with the spec implementation.
    fn vector(h: &Harness, scenario: &Scenario) -> Json {
        let env = h.env(&scenario.spend, scenario.index);
        let ctx = SighashContext {
            tx: &scenario.spend.tx,
            utxos: &scenario.spend.utxos,
            index: scenario.index as u32,
            control_block: env.control_block(),
            cmr: h.program.get_cmr(),
            genesis: scenario.spend.network.genesis_block_hash(),
        };

        let results: Vec<Json> = spec::MODES
            .iter()
            .map(|&mode| {
                json!({
                    "mode": mode_hex(mode),
                    "mode_hash": spec::mode_hash(&ctx, mode).map(|m| hex(&m)),
                    "sighash": spec::sighash(&ctx, mode).map(|m| hex(&m)),
                })
            })
            .collect();

        json!({
            "name": scenario.name,
            "description": scenario.description,
            "tx": hex(&serialize(ctx.tx)),
            "utxos": ctx.utxos.iter().map(|u| hex(&serialize(u))).collect::<Vec<_>>(),
            "index": ctx.index,
            "control_block": hex(&ctx.control_block.serialize()),
            "cmr": hex(&ctx.cmr),
            "genesis": hex(&ctx.genesis.to_byte_array()),
            "tap_env": hex(&spec::tap_env(&ctx)),
            "results": results,
        })
    }

    fn generate(h: &Harness) -> Json {
        json!({
            "description": "UNAUDITED: use with caution. Test vectors for docs/sighash_elements.md. Byte strings are hex. \
                            `tx` and `utxos` are consensus-serialized Elements transactions and \
                            outputs; `genesis` is in internal byte order. `mode_hash` is null \
                            for ALL, and `sighash` is null where the spend is invalid.",
            "vectors": scenarios(h).iter().map(|s| vector(h, s)).collect::<Vec<_>>(),
        })
    }

    fn read_file() -> Json {
        let text = std::fs::read_to_string(FILE).expect("vectors file exists");
        serde_json::from_str(&text).expect("vectors file is valid JSON")
    }

    #[simplex::test]
    fn vectors_file_is_current(_context: simplex::TestContext) -> anyhow::Result<()> {
        let generated = generate(&Harness::new());

        if std::env::var("SIGHASH_VECTORS_WRITE").is_ok() {
            std::fs::write(FILE, serde_json::to_string_pretty(&generated)? + "\n")?;
        }
        assert_eq!(
            read_file(),
            generated,
            "{FILE} is out of date; rerun with SIGHASH_VECTORS_WRITE=1 if the change is intended"
        );
        Ok(())
    }

    #[simplex::test]
    fn vectors_reproduce_from_file(_context: simplex::TestContext) -> anyhow::Result<()> {
        // Recompute every vector from the serialized inputs in the file alone.
        for vector in read_file()["vectors"].as_array().unwrap() {
            let s = |key: &str| vector[key].as_str().unwrap().to_owned();
            let tx: Transaction = deserialize(&unhex(&s("tx")))?;
            let utxos: Vec<TxOut> = vector["utxos"]
                .as_array()
                .unwrap()
                .iter()
                .map(|u| deserialize(&unhex(u.as_str().unwrap())).unwrap())
                .collect();
            let control_block = ControlBlock::from_slice(&unhex(&s("control_block")))?;
            let ctx = SighashContext {
                tx: &tx,
                utxos: &utxos,
                index: vector["index"].as_u64().unwrap() as u32,
                control_block: &control_block,
                cmr: unhex(&s("cmr")).try_into().unwrap(),
                genesis: BlockHash::from_byte_array(unhex(&s("genesis")).try_into().unwrap()),
            };

            assert_eq!(hex(&spec::tap_env(&ctx)), s("tap_env"), "{}", s("name"));
            for result in vector["results"].as_array().unwrap() {
                let mode = u8::from_str_radix(&result["mode"].as_str().unwrap()[2..], 16)?;
                let expect = |key: &str| result[key].as_str().map(str::to_owned);
                assert_eq!(
                    spec::mode_hash(&ctx, mode).map(|m| hex(&m)),
                    expect("mode_hash"),
                    "{} {}",
                    s("name"),
                    mode_name(mode)
                );
                assert_eq!(
                    spec::sighash(&ctx, mode).map(|m| hex(&m)),
                    expect("sighash"),
                    "{} {}",
                    s("name"),
                    mode_name(mode)
                );
            }
        }
        Ok(())
    }

    #[simplex::test]
    fn spec_matches_contract(_context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let mut mismatches = Vec::new();

        for scenario in scenarios(&h) {
            let env = h.env(&scenario.spend, scenario.index);
            let ctx = SighashContext {
                tx: &scenario.spend.tx,
                utxos: &scenario.spend.utxos,
                index: scenario.index as u32,
                control_block: env.control_block(),
                cmr: h.program.get_cmr(),
                genesis: scenario.spend.network.genesis_block_hash(),
            };

            for mode in spec::MODES {
                let from_spec = spec::sighash(&ctx, mode);
                let from_contract = h.contract_sighash(&scenario.spend, scenario.index, mode);

                // rust-simplicity treats a one-item witness starting with 0x50 as an annex, which
                // Elements does not. Until rust-simplicity#384 is released, the contract run here
                // disagrees with the spec in the modes that hash every input's annex.
                let known_divergence = scenario.name == "one_item_witness_starting_0x50"
                    && matches!(mode, spec::ALL | spec::NONE | spec::SINGLE);

                if (from_spec == from_contract) == known_divergence {
                    mismatches.push(format!(
                        "{} {}: spec {:?}, contract {:?}{}",
                        scenario.name,
                        mode_name(mode),
                        from_spec.map(|m| hex(&m)),
                        from_contract.map(|m| hex(&m)),
                        if known_divergence {
                            " (expected to differ until rust-simplicity#384 is released)"
                        } else {
                            ""
                        }
                    ));
                }
            }
        }

        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        Ok(())
    }
}

/// Spends checked by Elements consensus itself.
///
/// The other tests run the contract with rust-simplicity's environment. These build real spends
/// on regtest, sign the message computed from the spec (`tests/common/sighash_spec.rs`), and let
/// `elementsd` validate them, so they also check the spec against what Elements computes and
/// that every mode fits in a spend's cost budget.
mod consensus {
    use super::*;

    use simplex::simplicityhl::elements::OutPoint as ElementsOutPoint;
    use simplex::simplicityhl::elements::pset::{Input as PsetInput, Output as PsetOutput};

    use common::sighash_spec::{self as spec, SighashContext};

    const FUNDING: u64 = 20_000;
    const FEE: u64 = 2_000;

    /// A spend of a freshly funded contract UTXO, signed in `mode`, before it is broadcast.
    struct SignedSpend {
        tx: Transaction,
    }

    /// Fund the contract and return the new UTXO.
    fn fund(
        context: &simplex::TestContext,
        h: &Harness,
    ) -> anyhow::Result<(ElementsOutPoint, TxOut)> {
        let script = h.program.get_script_pubkey(context.get_network());
        let funding_txid = context
            .get_default_signer()
            .send(script.clone(), FUNDING)?
            .txid();
        let utxo = context
            .get_default_provider()
            .fetch_scripthash_utxos(&script)?
            .into_iter()
            .find(|utxo| utxo.outpoint.txid == funding_txid)
            .expect("the funding output is visible");
        Ok((utxo.outpoint, utxo.txout))
    }

    /// Build a spend of the contract UTXO paying `FUNDING - FEE` to the wallet, sign it in `mode`
    /// with the message from the spec, and finalize the contract's witness. `customize` can add
    /// to the input and outputs before signing.
    fn sign_spend(
        context: &simplex::TestContext,
        h: &Harness,
        mode: u8,
        customize: impl Fn(&mut PsetInput, &mut Vec<TxOut>),
    ) -> anyhow::Result<SignedSpend> {
        let network = context.get_network();
        let asset = network.policy_asset();
        let (outpoint, utxo) = fund(context, h)?;

        let mut input = PsetInput::from_prevout(outpoint);
        input.witness_utxo = Some(utxo.clone());
        let mut outputs = vec![
            TxOut {
                asset: Asset::Explicit(asset),
                value: Value::Explicit(FUNDING - FEE),
                nonce: Nonce::Null,
                script_pubkey: context.get_default_signer().get_address().script_pubkey(),
                witness: TxOutWitness::default(),
            },
            TxOut::new_fee(FEE, asset),
        ];
        customize(&mut input, &mut outputs);

        let mut pst = PartiallySignedTransaction::new_v2();
        pst.add_input(input);
        for output in outputs {
            pst.add_output(PsetOutput::from_txout(output));
        }
        let mut tx = pst.extract_tx()?;

        let utxos = [utxo];
        let spend = Spend {
            tx: tx.clone(),
            utxos: utxos.to_vec(),
            network: *network,
        };
        let control_block = h.control_block(&spend, 0);
        let ctx = SighashContext {
            tx: &tx,
            utxos: &utxos,
            index: 0,
            control_block: &control_block,
            cmr: h.program.get_cmr(),
            genesis: network.genesis_block_hash(),
        };
        let message = spec::sighash(&ctx, mode).expect("the spend is valid in this mode");
        let sig = Secp256k1::new()
            .sign_schnorr_no_aux_rand(&Message::from_digest(message), &h.keypair)
            .serialize();

        let witness = SighashElementsTestWitness {
            mode,
            pubkey: h.pubkey(),
            sig,
        };
        tx.input[0].witness.script_witness =
            h.program
                .as_ref()
                .finalize(&pst, &witness.build_witness(), 0, network)?;
        Ok(SignedSpend { tx })
    }

    fn broadcast(context: &simplex::TestContext, tx: &Transaction) -> Result<(), String> {
        context
            .get_default_provider()
            .broadcast_transaction(tx)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[simplex::test]
    fn every_mode_is_accepted(context: simplex::TestContext) -> anyhow::Result<()> {
        let h = Harness::new();
        let mut failures = Vec::new();

        for mode in spec::MODES {
            let spend = sign_spend(&context, &h, mode, |_, _| {})?;
            if let Err(e) = broadcast(&context, &spend.tx) {
                failures.push(format!("{}: {e}", mode_name(mode)));
            }
        }

        assert!(failures.is_empty(), "{}", failures.join("\n"));
        Ok(())
    }

    #[simplex::test]
    fn elementsd_enforces_what_each_mode_signs(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        // After signing, move one sat from the paired output to the fee. Only the modes that
        // sign no outputs may still be accepted.
        let h = Harness::new();
        let mut mismatches = Vec::new();

        for mode in spec::MODES {
            let mut spend = sign_spend(&context, &h, mode, |_, _| {})?;
            set_value(&mut spend.tx.output[0], FUNDING - FEE - 1);
            set_value(&mut spend.tx.output[1], FEE + 1);

            let accepted = broadcast(&context, &spend.tx).is_ok();
            let expected = NO_OUTPUTS.contains(&mode);
            if accepted != expected {
                mismatches.push(format!(
                    "{}: {} after changing the paired output",
                    mode_name(mode),
                    if accepted { "accepted" } else { "rejected" }
                ));
            }
        }

        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        Ok(())
    }

    #[simplex::test]
    fn issuance_on_signed_input_is_accepted(context: simplex::TestContext) -> anyhow::Result<()> {
        // A new issuance on the contract's input, so every mode hashes it.
        let h = Harness::new();
        let mut failures = Vec::new();

        for mode in spec::MODES {
            let spend = sign_spend(&context, &h, mode, |input, outputs| {
                input.issuance_value_amount = Some(1_000);
                input.issuance_asset_entropy = Some([mode; 32]);
                let (issued, _) = input.issuance_ids();
                outputs.push(TxOut {
                    asset: Asset::Explicit(issued),
                    value: Value::Explicit(1_000),
                    nonce: Nonce::Null,
                    script_pubkey: Script::new_op_return(b"issued"),
                    witness: TxOutWitness::default(),
                });
            })?;
            if let Err(e) = broadcast(&context, &spend.tx) {
                failures.push(format!("{}: {e}", mode_name(mode)));
            }
        }

        assert!(failures.is_empty(), "{}", failures.join("\n"));
        Ok(())
    }
}
