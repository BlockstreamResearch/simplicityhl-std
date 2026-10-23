//! Tests for `secp256k1::ecdsa`. Most run the contract locally on the bit machine; the regtest
//! spends check that elementsd accepts it, cost included.

use num_bigint::BigUint;
use rand::{RngCore, rngs::OsRng};
use secp256k1_zkp::ecdsa::Signature;
use secp256k1_zkp::{
    Message, PublicKey, Scalar, Secp256k1, SecretKey, rand::rngs::OsRng as SecpOsRng,
};
use serde_json::Value;

use simplex::program::{ArgumentsTrait, WitnessTrait};
use simplex::simplicityhl::ast::ElementsJetHinter;
use simplex::simplicityhl::dummy_env;
use simplex::simplicityhl::elements::hashes::{Hash, sha256};
use simplex::simplicityhl::elements::hex::FromHex;
use simplex::simplicityhl::simplicity::BitMachine;
use simplex::simplicityhl::simplicity::bit_machine::ExecutionError;
use simplex::simplicityhl::{CompiledProgram, UnstableFeatures};

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::secp256k1::ecdsa::EcdsaProgram;
use simplicityhl_std::artifacts::tests::secp256k1::ecdsa::derived_ecdsa::{
    EcdsaArguments, EcdsaWitness,
};

const WYCHEPROOF_P1363: &str =
    include_str!("../../vectors/wycheproof/ecdsa_secp256k1_sha256_p1363_test.json");

/// An oracle statement, signed as the SHA-256 of its text.
const ORACLE_STATEMENT: &str =
    "v1|PRICE|BTCUSD|84400.00|USD|2|binance,bitstamp,coinbase,kraken,okx|median|1741514700|482913";
const ORACLE_SIGNATURE_DER: &str = "3045022100a11d937449f9effd7ad15ffc84ad76e1ecb41c959871e305df5edbe0ebf59c67022071067f6d5619d6a19afc19b653000b6848bd8425537137287c0935fa6b1edf45";
const ORACLE_PUBKEY: &str = "02917e9826b0e7a61f695a5bb1a37714d1d742da025f4127c48892f3e4dbce8ba0";

enum FunctionToTest {
    EcdsaIsValid,
    EcdsaVerify,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
const SECP_P: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFC, 0x2F,
];

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
const SECP_N: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B, 0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

/// `(parity of y, x)`
type Point = (u8, [u8; 32]);
/// `(r, s)`
type Sig = ([u8; 32], [u8; 32]);

fn program() -> EcdsaProgram {
    EcdsaProgram::new(&EcdsaArguments {})
}

fn build_witness(function: u8, pk: Point, msg: [u8; 32], sig: Sig, expected: bool) -> EcdsaWitness {
    EcdsaWitness {
        function_index: function,
        pubkey: pk,
        msg,
        sig,
        expected,
    }
}

fn secp_p() -> BigUint {
    BigUint::from_bytes_be(&SECP_P)
}
fn secp_n() -> BigUint {
    BigUint::from_bytes_be(&SECP_N)
}

fn to_32_be(x: &BigUint) -> [u8; 32] {
    let b = x.to_bytes_be();

    let mut out = [0u8; 32];
    out[32 - b.len()..].copy_from_slice(&b);

    out
}

fn random_bytes() -> [u8; 32] {
    let mut b = [0u8; 32];
    OsRng.fill_bytes(&mut b);

    b
}

/// Uniform in `[1, n - 1]`.
fn random_scalar() -> BigUint {
    BigUint::from_bytes_be(&SecretKey::new(&mut SecpOsRng).secret_bytes())
}

fn inv_mod_n(a: &BigUint) -> BigUint {
    let n = secp_n();
    a.modpow(&(&n - 2u32), &n)
}

fn tweak(a: &BigUint) -> Scalar {
    Scalar::from_be_bytes(to_32_be(a)).unwrap()
}

fn secret_key(d: &BigUint) -> SecretKey {
    SecretKey::from_slice(&to_32_be(d)).unwrap()
}

fn public_key(d: &BigUint) -> PublicKey {
    PublicKey::from_secret_key(&Secp256k1::new(), &secret_key(d))
}

/// The point with x-coordinate `x` and even y, if any.
fn point_with_x(x: &BigUint) -> Option<PublicKey> {
    let mut compressed = vec![0x02];
    compressed.extend(to_32_be(x));
    PublicKey::from_slice(&compressed).ok()
}

fn compress(pk: &PublicKey) -> Point {
    let ser = pk.serialize();
    (ser[0] & 1, ser[1..].try_into().unwrap())
}

fn x_mod_n(pk: &PublicKey) -> BigUint {
    BigUint::from_bytes_be(&compress(pk).1) % secp_n()
}

fn split_sig(sig: &[u8; 64]) -> Sig {
    (sig[..32].try_into().unwrap(), sig[32..].try_into().unwrap())
}

fn flip_bit(mut bytes: [u8; 32], bit: usize) -> [u8; 32] {
    bytes[bit / 8] ^= 1 << (bit % 8);
    bytes
}

struct Signed {
    pk: Point,
    msg: [u8; 32],
    sig: Sig,
}

/// A random digest signed by libsecp256k1 with the key `d`.
fn sign_with_key(d: &BigUint) -> Signed {
    let msg = random_bytes();
    let sig = Secp256k1::new().sign_ecdsa(&Message::from_digest(msg), &secret_key(d));

    Signed {
        pk: compress(&public_key(d)),
        msg,
        sig: split_sig(&sig.serialize_compact()),
    }
}

fn sign_random() -> Signed {
    sign_with_key(&random_scalar())
}

/// A signature with the given `s`, over the digest `e = s * k - r * d`.
fn sign_with_s(s: &BigUint) -> Signed {
    let n = secp_n();
    let (d, k) = (random_scalar(), random_scalar());
    let r = x_mod_n(&public_key(&k));
    let e = (s * &k + &n - &r * &d % &n) % &n;

    Signed {
        pk: compress(&public_key(&d)),
        msg: to_32_be(&e),
        sig: (to_32_be(&r), to_32_be(s)),
    }
}

/// A signature with `u1 * G = u2 * Q`: the digest `e = r * d`, so `s = 2 * e / k`.
fn sign_doubling(d: &BigUint) -> Signed {
    let n = secp_n();
    let k = random_scalar();
    let r = x_mod_n(&public_key(&k));
    let e = &r * d % &n;
    let s = &e * 2u32 * inv_mod_n(&k) % &n;

    Signed {
        pk: compress(&public_key(d)),
        msg: to_32_be(&e),
        sig: (to_32_be(&r), to_32_be(&s)),
    }
}

/// A signature with `u1 * G = -u2 * Q`, so `R` is infinity: the digest `e = -r * d`.
fn cancelling(d: &BigUint) -> Signed {
    let n = secp_n();
    let (r, s) = (random_scalar(), random_scalar());
    let e = (&n - &r * d % &n) % &n;

    Signed {
        pk: compress(&public_key(d)),
        msg: to_32_be(&e),
        sig: (to_32_be(&r), to_32_be(&s)),
    }
}

/// The digest and signature for which verification under any key computes
/// `R = a * G + b * Q`: `r = R.x`, `s = r / b`, `e = a * s`. No secret key needed.
fn signature_for(big_r: &PublicKey, a: &BigUint, b: &BigUint) -> ([u8; 32], Sig) {
    let n = secp_n();
    let r = x_mod_n(big_r);
    let s = &r * inv_mod_n(b) % &n;
    let e = a * &s % &n;

    (to_32_be(&e), (to_32_be(&r), to_32_be(&s)))
}

/// A digest and signature that verify under `q`, made without its secret key.
fn forge_for(q: &PublicKey) -> ([u8; 32], Sig) {
    let (a, b) = (random_scalar(), random_scalar());
    let big_r = public_key(&a)
        .combine(&q.mul_tweak(&Secp256k1::new(), &tweak(&b)).unwrap())
        .unwrap();

    signature_for(&big_r, &a, &b)
}

/// A key, digest and signature with `R.x >= n`, made by picking `R` and solving
/// `Q = (R - a * G) / b`. Also returns `R.x`.
fn forge_with_large_x() -> (Point, [u8; 32], Sig, [u8; 32]) {
    let secp = Secp256k1::new();
    let big_r = (1u32..)
        .find_map(|c| point_with_x(&(secp_n() + c)))
        .unwrap();
    let (a, b) = (random_scalar(), random_scalar());
    let q = big_r
        .combine(&public_key(&a).negate(&secp))
        .unwrap()
        .mul_tweak(&secp, &tweak(&inv_mod_n(&b)))
        .unwrap();
    let (msg, sig) = signature_for(&big_r, &a, &b);

    (compress(&q), msg, sig, compress(&big_r).1)
}

/// libsecp256k1's verdict, after normalizing to the low-S form it requires.
fn libsecp_accepts(pk: Point, msg: [u8; 32], sig: Sig) -> bool {
    let pk = PublicKey::from_slice(&[&[0x02 | pk.0][..], &pk.1[..]].concat());
    let sig = Signature::from_compact(&[sig.0, sig.1].concat());
    let (Ok(pk), Ok(mut sig)) = (pk, sig) else {
        return false;
    };
    sig.normalize_s();

    Secp256k1::verification_only()
        .verify_ecdsa(&Message::from_digest(msg), &sig, &pk)
        .is_ok()
}

#[derive(Debug)]
enum Outcome {
    Ok,
    AssertFailed,
    Panicked,
}

/// Runs the contract on the bit machine, in a dummy environment.
struct Harness {
    compiled: CompiledProgram,
}

impl Harness {
    fn new() -> Self {
        // Flattened library imports need the `imports` feature.
        let compiled = CompiledProgram::new_with_unstable(
            EcdsaProgram::SOURCE,
            &UnstableFeatures::all(),
            EcdsaArguments {}.build_arguments(),
            false,
            Box::new(ElementsJetHinter),
        )
        .unwrap();

        Self { compiled }
    }

    fn run(&self, witness: EcdsaWitness) -> Outcome {
        let satisfied = self.compiled.satisfy(witness.build_witness()).unwrap();
        let redeem = satisfied.redeem();

        match BitMachine::for_program(redeem)
            .unwrap()
            .exec(redeem, &dummy_env::dummy())
        {
            Ok(_) => Outcome::Ok,
            Err(ExecutionError::JetFailed(_)) => Outcome::AssertFailed,
            Err(ExecutionError::ReachedFailNode(_)) => Outcome::Panicked,
            Err(e) => panic!("unexpected execution error: {e}"),
        }
    }

    /// What `ecdsa_is_valid` returns. Exactly one of `expected = true` and `false` must pass,
    /// or it failed instead of returning.
    fn is_valid(&self, pk: Point, msg: [u8; 32], sig: Sig) -> bool {
        let is_valid = op(FunctionToTest::EcdsaIsValid);

        match (
            self.run(build_witness(is_valid, pk, msg, sig, true)),
            self.run(build_witness(is_valid, pk, msg, sig, false)),
        ) {
            (Outcome::Ok, Outcome::AssertFailed) => true,
            (Outcome::AssertFailed, Outcome::Ok) => false,
            outcomes => panic!("ecdsa_is_valid did not return a value: {outcomes:?}"),
        }
    }
}

fn hex_str(value: &Value) -> Vec<u8> {
    Vec::from_hex(value.as_str().unwrap()).unwrap()
}

#[simplex::test]
fn wycheproof_p1363_vectors(_context: simplex::TestContext) -> anyhow::Result<()> {
    let file: Value = serde_json::from_str(WYCHEPROOF_P1363)?;
    let harness = Harness::new();
    let (mut ran, mut skipped) = (0, 0);
    let mut failures = Vec::new();

    for group in file["testGroups"].as_array().unwrap() {
        assert_eq!(group["sha"], "SHA-256");
        let pk = PublicKey::from_slice(&hex_str(&group["publicKey"]["uncompressed"]))?;

        for test in group["tests"].as_array().unwrap() {
            let expected = match test["result"].as_str().unwrap() {
                "valid" => true,
                "invalid" => false,
                other => panic!("unexpected result {other:?}"),
            };

            // Other lengths are invalid, and have no `(Scalar, Scalar)` form.
            let Ok(sig) = <[u8; 64]>::try_from(hex_str(&test["sig"])) else {
                assert!(!expected, "tcId {}", test["tcId"]);
                skipped += 1;
                continue;
            };

            let msg = sha256::Hash::hash(&hex_str(&test["msg"])).to_byte_array();
            if harness.is_valid(compress(&pk), msg, split_sig(&sig)) != expected {
                failures.push(format!("tcId {} ({})", test["tcId"], test["comment"]));
            }
            ran += 1;
        }
    }

    assert!(failures.is_empty(), "failed:\n{}", failures.join("\n"));
    assert_eq!((ran, skipped), (234, 18));

    Ok(())
}

#[simplex::test]
fn agrees_with_libsecp256k1(_context: simplex::TestContext) -> anyhow::Result<()> {
    // Mutate a valid signature, key or digest, and compare verdicts.
    let harness = Harness::new();

    for case in 0..160 {
        let Signed {
            mut pk,
            mut msg,
            mut sig,
        } = sign_random();
        let bit = OsRng.next_u32() as usize % 256;

        match case % 10 {
            0 => {}
            1 => msg = flip_bit(msg, bit),
            2 => sig.0 = flip_bit(sig.0, bit),
            3 => sig.1 = flip_bit(sig.1, bit),
            4 => pk.1 = flip_bit(pk.1, bit),
            5 => pk.0 ^= 1,
            6 => sig.1 = to_32_be(&(secp_n() - BigUint::from_bytes_be(&sig.1))),
            7 => sig = (random_bytes(), random_bytes()),
            8 => pk.1 = random_bytes(),
            _ => msg = random_bytes(),
        }

        assert_eq!(
            harness.is_valid(pk, msg, sig),
            libsecp_accepts(pk, msg, sig),
            "case {case}: pk {pk:x?}, msg {msg:x?}, sig {sig:x?}"
        );
    }

    Ok(())
}

#[simplex::test]
fn accepts_keys_g_and_minus_g(_context: simplex::TestContext) -> anyhow::Result<()> {
    // They share G's x-coordinate, which some implementations mishandle.
    let harness = Harness::new();

    for d in [BigUint::from(1u8), secp_n() - 1u8] {
        let Signed { pk, msg, sig } = sign_with_key(&d);
        assert!(harness.is_valid(pk, msg, sig), "d = {d:x}");
    }

    Ok(())
}

#[simplex::test]
fn accepts_high_s(_context: simplex::TestContext) -> anyhow::Result<()> {
    // Unlike libsecp256k1, the contract accepts (r, n - s).
    let harness = Harness::new();
    let Signed { pk, msg, sig } = sign_random();
    let high_s = to_32_be(&(secp_n() - BigUint::from_bytes_be(&sig.1)));

    assert!(harness.is_valid(pk, msg, (sig.0, high_s)));

    Ok(())
}

#[simplex::test]
fn rejects_psychic_signatures(_context: simplex::TestContext) -> anyhow::Result<()> {
    // CVE-2022-21449: Java accepted (0, 0) for any key and message.
    let harness = Harness::new();
    let Signed { pk, msg, sig } = sign_random();
    let zero = [0u8; 32];

    for (r, s) in [
        (zero, zero),
        (zero, sig.1),
        (sig.0, zero),
        (SECP_N, sig.1),
        (sig.0, SECP_N),
    ] {
        assert!(!harness.is_valid(pk, msg, (r, s)), "r {r:x?}, s {s:x?}");
    }

    Ok(())
}

#[simplex::test]
fn checks_the_range_of_s(_context: simplex::TestContext) -> anyhow::Result<()> {
    // s + n fits in 256 bits when s < 2^256 - n: try both ends of that range, and n - 1.
    let harness = Harness::new();
    let s_bound = (BigUint::from(1u8) << 256) - secp_n();

    for s in [BigUint::from(1u8), &s_bound - 1u8, secp_n() - 1u8] {
        let Signed { pk, msg, sig } = sign_with_s(&s);
        assert!(libsecp_accepts(pk, msg, sig), "s = {s:x}");
        assert!(harness.is_valid(pk, msg, sig), "s = {s:x}");

        let s_plus_n = &s + secp_n();
        if s_plus_n.bits() <= 256 {
            let sig = (sig.0, to_32_be(&s_plus_n));
            assert!(!harness.is_valid(pk, msg, sig), "s = {s:x}");
        }
    }

    Ok(())
}

#[simplex::test]
fn accepts_r_from_an_x_coordinate_above_n(_context: simplex::TestContext) -> anyhow::Result<()> {
    // Then r = R.x - n, and r = R.x must fail.
    let harness = Harness::new();
    let (pk, msg, sig, x) = forge_with_large_x();

    assert!(BigUint::from_bytes_be(&x) >= secp_n());
    assert!(libsecp_accepts(pk, msg, sig));
    assert!(harness.is_valid(pk, msg, sig));
    assert!(!harness.is_valid(pk, msg, (x, sig.1)));

    Ok(())
}

#[simplex::test]
fn accepts_doubling_in_the_linear_combination(
    _context: simplex::TestContext,
) -> anyhow::Result<()> {
    // u1 * G = u2 * Q breaks incomplete addition formulas.
    let harness = Harness::new();

    for d in [BigUint::from(1u8), secp_n() - 1u8, random_scalar()] {
        let Signed { pk, msg, sig } = sign_doubling(&d);
        assert!(libsecp_accepts(pk, msg, sig), "d = {d:x}");
        assert!(harness.is_valid(pk, msg, sig), "d = {d:x}");
    }

    Ok(())
}

#[simplex::test]
fn rejects_infinity_in_the_linear_combination(
    _context: simplex::TestContext,
) -> anyhow::Result<()> {
    // u1 * G = -u2 * Q leaves no x-coordinate to compare with r.
    let harness = Harness::new();

    for d in [BigUint::from(1u8), secp_n() - 1u8, random_scalar()] {
        let Signed { pk, msg, sig } = cancelling(&d);
        assert!(!libsecp_accepts(pk, msg, sig), "d = {d:x}");
        assert!(!harness.is_valid(pk, msg, sig), "d = {d:x}");
    }

    Ok(())
}

#[simplex::test]
fn reduces_the_digest_mod_n(_context: simplex::TestContext) -> anyhow::Result<()> {
    // h >= n verifies like h - n.
    let harness = Harness::new();
    let n = secp_n();
    let d = random_scalar();
    let h = &n + BigUint::from_bytes_be(&random_bytes()) % ((BigUint::from(1u8) << 256) - &n);

    let sig = Secp256k1::new().sign_ecdsa(&Message::from_digest(to_32_be(&h)), &secret_key(&d));
    let (pk, sig) = (
        compress(&public_key(&d)),
        split_sig(&sig.serialize_compact()),
    );

    assert!(harness.is_valid(pk, to_32_be(&h), sig));
    assert!(harness.is_valid(pk, to_32_be(&(&h - &n)), sig));

    Ok(())
}

#[simplex::test]
fn anyone_can_sign_a_digest_they_choose(_context: simplex::TestContext) -> anyhow::Result<()> {
    // Why the contract must compute `msg`.
    let harness = Harness::new();
    let q = public_key(&random_scalar());
    let (msg, sig) = forge_for(&q);

    assert!(harness.is_valid(compress(&q), msg, sig));

    Ok(())
}

#[simplex::test]
fn accepts_an_oracle_statement(_context: simplex::TestContext) -> anyhow::Result<()> {
    let harness = Harness::new();
    let pk = compress(&PublicKey::from_slice(
        &Vec::from_hex(ORACLE_PUBKEY).unwrap(),
    )?);
    let der = Vec::from_hex(ORACLE_SIGNATURE_DER).unwrap();
    let sig = split_sig(&Signature::from_der(&der)?.serialize_compact());
    let digest = |statement: &str| sha256::Hash::hash(statement.as_bytes()).to_byte_array();
    let other_price = ORACLE_STATEMENT.replace("84400.00", "94400.00");

    assert!(harness.is_valid(pk, digest(ORACLE_STATEMENT), sig));
    assert!(!harness.is_valid(pk, digest(&other_price), sig));

    Ok(())
}

#[simplex::test]
fn rejects_unreduced_public_key(_context: simplex::TestContext) -> anyhow::Result<()> {
    // x + p decodes like x, and fits in 256 bits for a tiny x. Its key is unknown, so forge.
    let harness = Harness::new();
    let q = (1u32..).find_map(|x| point_with_x(&x.into())).unwrap();
    let (msg, sig) = forge_for(&q);
    let pk = compress(&q);
    let x_plus_p = to_32_be(&(BigUint::from_bytes_be(&pk.1) + secp_p()));

    assert!(libsecp_accepts(pk, msg, sig));
    assert!(harness.is_valid(pk, msg, sig));
    assert!(!harness.is_valid((pk.0, x_plus_p), msg, sig));

    Ok(())
}

#[simplex::test]
fn rejects_public_keys_off_the_curve(_context: simplex::TestContext) -> anyhow::Result<()> {
    // Some libraries take x = 0 as infinity, and then R = u1 * G, which this signature meets.
    let harness = Harness::new();
    let a = random_scalar();
    let (msg, sig) = signature_for(&public_key(&a), &a, &random_scalar());
    let off_curve = (1u32..)
        .find(|&x| point_with_x(&x.into()).is_none())
        .unwrap();

    for x in [0, off_curve] {
        for parity in [0, 1] {
            let pk = (parity, to_32_be(&x.into()));
            assert!(!harness.is_valid(pk, msg, sig), "x = {x}, parity {parity}");
        }
    }

    Ok(())
}

// Spends on regtest

#[simplex::test]
fn ecdsa_verify_spend_with_valid_signature(context: simplex::TestContext) -> anyhow::Result<()> {
    let Signed { pk, msg, sig } = sign_random();

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::EcdsaVerify), pk, msg, sig, false),
        Expect::Ok,
    )
}

#[simplex::test]
fn ecdsa_verify_spend_with_invalid_signature(context: simplex::TestContext) -> anyhow::Result<()> {
    let Signed { pk, sig, .. } = sign_random();
    let other = sign_random();

    run(
        &context,
        program(),
        build_witness(op(FunctionToTest::EcdsaVerify), pk, other.msg, sig, false),
        Expect::AssertFailed,
    )
}
