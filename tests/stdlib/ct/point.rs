use num_bigint::BigUint;
use secp256k1_zkp::{PublicKey, SecretKey};

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::ct::point::PointProgram as CtPointTestProgram;
use simplicityhl_std::artifacts::tests::ct::point::derived_point::{
    PointArguments as CtPointTestArguments, PointWitness as CtPointTestWitness,
};

use super::helpers::{
    ConfPoint, Ge, blinded_generator, commitment, conf_point, decode, random_amount,
    random_asset_id, random_scalar, scalar, secp, unblinded_generator,
};

use FunctionToTest::*;

enum FunctionToTest {
    ConfPointToGe,
    ConfPointToGej,
}

fn program() -> CtPointTestProgram {
    CtPointTestProgram::new(&CtPointTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: CtPointTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: CtPointTestWitness {
            function_index: function as u8,
            point: (0, [0; 32]),
            expected_ge: ([0; 32], [0; 32]),
        },
    }
}

impl Case {
    /// `point`: a confidential point in the Elements encoding.
    fn point(mut self, point: ConfPoint) -> Self {
        self.witness.point = point;
        self
    }

    /// `expected_ge`: the affine point the arm should decode to.
    fn expect_ge(mut self, expected_ge: Ge) -> Self {
        self.witness.expected_ge = expected_ge;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        run(context, program(), self.witness, expect)
    }
}

fn multiple_of_g(k: u8) -> (ConfPoint, Ge) {
    let vbf = BigUint::from(k);
    let c = conf_point(commitment(0, unblinded_generator([0; 32]), &vbf).serialize());

    let sk = SecretKey::from_slice(&scalar(&vbf)).unwrap();
    let pk = PublicKey::from_secret_key(&secp(), &sk).serialize_uncompressed();

    (
        c,
        (pk[1..33].try_into().unwrap(), pk[33..].try_into().unwrap()),
    )
}

// conf_point_to_ge
#[simplex::test]
fn conf_point_to_ge_square_y(context: simplex::TestContext) -> anyhow::Result<()> {
    let (point, ge) = multiple_of_g(6);

    case(ConfPointToGe).point(point).expect_ge(ge).run(&context)
}

#[simplex::test]
fn conf_point_to_ge_non_square_y(context: simplex::TestContext) -> anyhow::Result<()> {
    let (point, ge) = multiple_of_g(3);

    case(ConfPointToGe).point(point).expect_ge(ge).run(&context)
}

#[simplex::test]
fn conf_point_to_ge_asset_generator(context: simplex::TestContext) -> anyhow::Result<()> {
    let h = blinded_generator(random_asset_id(), &random_scalar()).serialize();

    case(ConfPointToGe)
        .point(conf_point(h))
        .expect_ge(decode(h))
        .run(&context)
}

#[simplex::test]
fn conf_point_to_ge_rejects_flipped_bit(context: simplex::TestContext) -> anyhow::Result<()> {
    let (point, ge) = multiple_of_g(6);

    case(ConfPointToGe)
        .point((point.0 ^ 1, point.1))
        .expect_ge(ge)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn conf_point_to_ge_rejects_x_off_curve(context: simplex::TestContext) -> anyhow::Result<()> {
    // x = 5: 5^3 + 7 = 132 has no square root modulo p.
    let mut x = [0u8; 32];
    x[31] = 5;

    case(ConfPointToGe)
        .point((0, x))
        .expecting(&context, Expect::PrunedBranch)
}

// conf_point_to_gej
#[simplex::test]
fn conf_point_to_gej_value_commitment(context: simplex::TestContext) -> anyhow::Result<()> {
    let generator = blinded_generator(random_asset_id(), &random_scalar());
    let c = commitment(random_amount(), generator, &random_scalar()).serialize();

    case(ConfPointToGej)
        .point(conf_point(c))
        .expect_ge(decode(c))
        .run(&context)
}
