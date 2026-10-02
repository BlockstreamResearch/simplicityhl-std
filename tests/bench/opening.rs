//! "To verify an asset commitment, it'd be faster to take an exact value proof
//! … rather than to take an abf and vbf": `simf/bench/ct/opening.simf`.
//!
//! The proof needs `(v+1)*H_0`, so the answer depends on the cost of `v*H`
//! (see `vh`). Candidates 5 and 6 are given `H_0` and `-(v+1)*H_0` to find the
//! break-even cost of `(v+1)*H_0`.

use num_bigint::BigUint;
use secp256k1_zkp::{Keypair, Message, SecretKey};

use simplicityhl_std::artifacts::bench::ct::opening::OpeningProgram;
use simplicityhl_std::artifacts::bench::ct::opening::derived_opening::OpeningWitness;

use crate::helpers::{
    blinded_generator, commitment, conf_point, decode, random_amount, random_asset_id,
    random_scalar, scalar, secp, total_blinding, unblinded_generator,
};
use crate::{Cost, ge, mul, print_header, print_row, run, sig_all_hash};

/// Function indices in `opening.simf`; 0 is the baseline.
const CANDIDATES: [(u8, &str); 4] = [
    (1, "abf, vbf: assert_opens_to"),
    (2, "abf, vbf: former assert_opens_to"),
    (3, "exact value proof, scale"),
    (4, "exact value proof, double-and-add"),
];

/// Unsound candidates that are given `H_0` and `-(v+1)*H_0`.
const GIVEN: [(u8, &str); 2] = [
    (5, "abf, vbf: linear verifies, H0 given"),
    (6, "exact value proof, (v+1)*H0 given"),
];

/// The BIP-340 signature `assert_opens_to_proof` expects: by `secret` over the
/// environment's `sig_all_hash`.
fn sign(secret: &BigUint) -> [u8; 64] {
    let keypair =
        Keypair::from_secret_key(&secp(), &SecretKey::from_slice(&scalar(secret)).unwrap());
    let message = Message::from_digest(sig_all_hash());

    secp()
        .sign_schnorr_no_aux_rand(&message, &keypair)
        .serialize()
}

/// A confidential output holding `v` of a random asset, with both openings.
fn witness(index: u8, v: u64) -> OpeningWitness {
    let (asset_id, abf, vbf) = (random_asset_id(), random_scalar(), random_scalar());
    let generator = blinded_generator(asset_id, &abf);
    let h0 = decode(unblinded_generator(asset_id).serialize());

    OpeningWitness {
        function_index: index,
        asset: conf_point(generator.serialize()),
        amount: conf_point(commitment(v, generator, &vbf).serialize()),
        asset_id,
        v,
        abf: scalar(&abf),
        vbf: scalar(&vbf),
        // The secret of `c + h - (v+1)*H_0` is `(v+1)*abf + vbf`.
        proof: sign(&total_blinding(v + 1, &abf, &vbf)),
        h0,
        neg_vh0: ge(mul(h0, v + 1).negate(&secp())),
    }
}

#[test]
fn candidates_are_correct() {
    for (index, name) in CANDIDATES {
        for v in [0, 1, random_amount()] {
            run(OpeningProgram::SOURCE, &witness(index, v))
                .unwrap_or_else(|e| panic!("{name} rejected a valid opening to {v}: {e}"));
        }

        let mut wrong_value = witness(index, 7);
        wrong_value.v = 8;
        assert!(
            run(OpeningProgram::SOURCE, &wrong_value).is_err(),
            "{name} accepted a wrong value"
        );

        let mut wrong_asset = witness(index, 7);
        wrong_asset.asset_id = random_asset_id();
        assert!(
            run(OpeningProgram::SOURCE, &wrong_asset).is_err(),
            "{name} accepted a wrong asset"
        );
    }
}

/// The unsound candidates must at least accept valid openings, or their cost
/// would be the cost of failing.
#[test]
fn given_candidates_accept() {
    for (index, name) in GIVEN {
        for v in [0, 1, random_amount()] {
            run(OpeningProgram::SOURCE, &witness(index, v))
                .unwrap_or_else(|e| panic!("{name} rejected a valid opening to {v}: {e}"));
        }
    }
}

#[test]
fn candidates_cost() {
    print_header("Verify a confidential `(asset, amount)` opening");

    let w = witness(0, random_amount());
    let base = run(OpeningProgram::SOURCE, &w).unwrap();
    let cost = |index: u8| -> Cost {
        let mut candidate = w.clone();
        candidate.function_index = index;
        run(OpeningProgram::SOURCE, &candidate).unwrap() - base
    };

    for (index, name) in CANDIDATES {
        print_row(name, &[cost(index)]);
    }
    for (index, name) in GIVEN {
        print_row(&format!("{name} (unsound)"), &[cost(index)]);
    }

    let (linear_verifies, proof) = (cost(GIVEN[0].0), cost(GIVEN[1].0));
    println!(
        "With H0 embedded, the proof wins only if `(v+1)*H0` costs less than {} mWU",
        linear_verifies.milliweight as i64 - proof.milliweight as i64
    );
}
