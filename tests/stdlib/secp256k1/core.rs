use num_bigint::BigUint;
use rand::{RngCore, rngs::OsRng};
use secp256k1_zkp::{PublicKey, SecretKey, rand::rngs::OsRng as SecpOsRng};

use crate::common::core::{Expect, run};
use crate::common::fuzz::DispatchFuzzCase;

use simplicityhl_std::artifacts::tests::secp256k1::operations::OperationsProgram as Secp256k1OperationsTestProgram;
use simplicityhl_std::artifacts::tests::secp256k1::operations::derived_operations::{
    OperationsArguments as Secp256k1OperationsTestArguments,
    OperationsWitness as Secp256k1OperationsTestWitness,
};

pub const ZERO: U256 = [0; 32];
pub const ONE: U256 = u256(1);

/// A big-endian 256-bit value.
pub type U256 = [u8; 32];
/// A **f**ield **e**lement, arithmetic modulo **p**.
pub type Fe = U256;
/// A multiplier for curve points, arithmetic modulo the group order **n**.
pub type Scalar = U256;
/// A **G**roup **E**lement, represented as an affine curve point (x, y).
pub type Ge = (Fe, Fe);
/// A **G**roup **E**lement, represented using **J**acobian coordinates ((X, Y), Z).
pub type Gej = (Ge, Fe);
/// A compressed curve point (y_parity, x).
pub type Point = (u8, Fe);

pub enum FunctionToTest {
    GeToPoint,
    PointToGej,
    FeSub,
    ScalarSub,
    GejSub,
    FeEq,
    ScalarEq,
    GeEq,
    GejPointEq,
    SafeGejNormalize,
}

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
pub const SECP_P: U256 = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFC, 0x2F,
];

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
pub const SECP_N: U256 = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B, 0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

pub const fn u256(value: u128) -> U256 {
    let mut out = [0; 32];
    let bytes = value.to_be_bytes();
    let mut i = 0;

    while i < 16 {
        out[16 + i] = bytes[i];
        i += 1;
    }

    out
}

pub fn big(value: U256) -> BigUint {
    BigUint::from_bytes_be(&value)
}

/// # Panics
/// If `value` does not fit in 256 bits.
pub fn to_u256(value: &BigUint) -> U256 {
    let bytes = value.to_bytes_be();
    assert!(bytes.len() <= 32, "{value:#x} does not fit in 256 bits");

    let mut out = [0; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}

/// `a + b` as plain integers.
///
/// # Panics
/// On overflow past 256 bits.
pub fn add_u256(a: U256, b: U256) -> U256 {
    to_u256(&(big(a) + big(b)))
}

/// `a - b (mod modulus)`, canonical. Operands may be non-canonical.
pub fn sub_mod(a: U256, b: U256, modulus: U256) -> U256 {
    let m = big(modulus);

    let a = big(a) % &m + &m;
    let b = big(b) % &m;

    to_u256(&((a - b) % &m))
}

pub fn fe_mul(a: Fe, b: Fe) -> Fe {
    to_u256(&(big(a) * big(b) % big(SECP_P)))
}

pub fn pk_to_ge(pk: &PublicKey) -> Ge {
    let serialized = pk.serialize_uncompressed(); // 0x04 || x(32) || y(32)

    (
        serialized[1..33].try_into().unwrap(),
        serialized[33..65].try_into().unwrap(),
    )
}

pub fn ge_to_gej(ge: Ge) -> Gej {
    (ge, ONE)
}

pub fn ge_negate((x, y): Ge) -> Ge {
    (x, sub_mod(ZERO, y, SECP_P))
}

pub fn compress((x, y): Ge) -> Point {
    (
        y[31] & 1, // 0 (even y) or 1 (odd y)
        x,
    )
}

/// `(λ²X, λ³Y, λ)`: the same point as `(X, Y, 1)` for any non-zero λ.
pub fn rescale((x, y): Ge, lambda: Fe) -> Gej {
    let lambda_squared = fe_mul(lambda, lambda);
    let lambda_cubed = fe_mul(lambda_squared, lambda);

    ((fe_mul(x, lambda_squared), fe_mul(y, lambda_cubed)), lambda)
}

fn random_mod(modulus: U256) -> U256 {
    let mut bytes = [0; 32];
    OsRng.fill_bytes(&mut bytes);

    to_u256(&(big(bytes) % big(modulus)))
}

pub fn random_fe() -> Fe {
    random_mod(SECP_P)
}

pub fn random_scalar() -> Scalar {
    random_mod(SECP_N)
}

pub fn random_public_key() -> PublicKey {
    let secret_key = SecretKey::new(&mut SecpOsRng);

    PublicKey::from_secret_key(secp256k1_zkp::SECP256K1, &secret_key)
}

pub fn random_ge() -> Ge {
    pk_to_ge(&random_public_key())
}

/// One dispatch arm of the contract, plus the witness it reads.
#[derive(Clone, Debug)]
pub struct Case {
    witness: Secp256k1OperationsTestWitness,
}

pub fn case(function: FunctionToTest) -> Case {
    Case {
        witness: Secp256k1OperationsTestWitness {
            function_index: function as u8,
            first_uint: U256::default(),
            second_uint: U256::default(),
            first_ge: Ge::default(),
            second_ge: Ge::default(),
            first_gej: Gej::default(),
            second_gej: Gej::default(),
            first_point: Point::default(),
            expected_uint: U256::default(),
            expected_ge: Ge::default(),
            expected_gej: Gej::default(),
            expected_point: Point::default(),
        },
    }
}

impl Case {
    /// The two field or scalar operands, `first_uint` and `second_uint`.
    pub fn uints(mut self, first_uint: U256, second_uint: U256) -> Self {
        self.witness.first_uint = first_uint;
        self.witness.second_uint = second_uint;
        self
    }

    /// Only `first_ge`.
    pub fn ge(mut self, first_ge: Ge) -> Self {
        self.witness.first_ge = first_ge;
        self
    }

    /// The two affine points, `first_ge` and `second_ge`.
    pub fn ges(mut self, first_ge: Ge, second_ge: Ge) -> Self {
        self.witness.first_ge = first_ge;
        self.witness.second_ge = second_ge;
        self
    }

    /// Only `first_gej`.
    pub fn gej(mut self, first_gej: Gej) -> Self {
        self.witness.first_gej = first_gej;
        self
    }

    /// The two Jacobian points, `first_gej` and `second_gej`.
    pub fn gejs(mut self, first_gej: Gej, second_gej: Gej) -> Self {
        self.witness.first_gej = first_gej;
        self.witness.second_gej = second_gej;
        self
    }

    /// `first_point`: a compressed point, parity byte and x coordinate.
    pub fn point(mut self, first_point: Point) -> Self {
        self.witness.first_point = first_point;
        self
    }

    /// `expected_uint`: the field or scalar the arm should produce.
    pub fn expect_uint(mut self, expected_uint: U256) -> Self {
        self.witness.expected_uint = expected_uint;
        self
    }

    /// `expected_ge`: the affine point the arm should produce.
    pub fn expect_ge(mut self, expected_ge: Ge) -> Self {
        self.witness.expected_ge = expected_ge;
        self
    }

    /// `expected_gej`: the Jacobian point the arm should produce.
    pub fn expect_gej(mut self, expected_gej: Gej) -> Self {
        self.witness.expected_gej = expected_gej;
        self
    }

    /// `expected_point`: the compressed point the arm should produce.
    pub fn expect_point(mut self, expected_point: Point) -> Self {
        self.witness.expected_point = expected_point;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    pub fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    pub fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        let program = Secp256k1OperationsTestProgram::new(Case::arguments());

        run(context, program, self.witness, expect)
    }
}

impl DispatchFuzzCase for Case {
    type Program = Secp256k1OperationsTestProgram;

    const PROGRAM_NAME: &'static str = "secp256k1::operations";

    fn arguments() -> Secp256k1OperationsTestArguments {
        Secp256k1OperationsTestArguments {}
    }

    fn into_witness(self) -> Secp256k1OperationsTestWitness {
        self.witness
    }
}

pub mod cases {
    use super::*;

    use FunctionToTest::*;

    /// `ge_to_point` keeps x and takes the parity of y.
    pub fn ge_to_point(ge: Ge) -> Case {
        case(GeToPoint).ge(ge).expect_point(compress(ge))
    }

    /// `point_to_gej` decompresses to the same point.
    pub fn point_to_gej(ge: Ge) -> Case {
        let point = compress(ge);

        case(PointToGej).point(point).expect_point(point)
    }

    /// `fe_sub` matches the reference, for any (also non-canonical) operands.
    pub fn fe_sub(a: Fe, b: Fe) -> Case {
        case(FeSub).uints(a, b).expect_uint(sub_mod(a, b, SECP_P))
    }

    /// `scalar_sub` matches the reference, for any (also non-canonical) operands.
    pub fn scalar_sub(a: Scalar, b: Scalar) -> Case {
        case(ScalarSub)
            .uints(a, b)
            .expect_uint(sub_mod(a, b, SECP_N))
    }

    /// `gej_sub` matches the reference, the point at infinity included (`p == q`).
    pub fn gej_sub(p: PublicKey, q: PublicKey) -> Case {
        let expected = if p == q {
            // The expected witness for infinity uses all-zero coordinates.
            Gej::default()
        } else {
            let difference = p
                .combine(&q.negate(secp256k1_zkp::SECP256K1))
                .expect("p - q non-infinity");

            ge_to_gej(pk_to_ge(&difference))
        };

        case(GejSub)
            .gejs(ge_to_gej(pk_to_ge(&p)), ge_to_gej(pk_to_ge(&q)))
            .expect_gej(expected)
    }

    /// `fe_eq(a, b)`, accepted iff `a ≡ b (mod p)`.
    pub fn fe_eq(a: Fe, b: Fe) -> Case {
        case(FeEq).uints(a, b)
    }

    /// `scalar_eq(a, b)`, accepted iff `a ≡ b (mod n)`.
    pub fn scalar_eq(a: Scalar, b: Scalar) -> Case {
        case(ScalarEq).uints(a, b)
    }

    /// `ge_eq(P, -P)`: must be rejected, y differs.
    pub fn ge_eq_negation(ge: Ge) -> Case {
        case(GeEq).ges(ge, ge_negate(ge))
    }

    /// `gej_point_eq` holds for any Jacobian representation `(λ²X, λ³Y, λ)` of P.
    pub fn gej_point_eq_rescaled(ge: Ge, lambda: Fe) -> Case {
        case(GejPointEq)
            .gej(rescale(ge, lambda))
            .point(compress(ge))
    }

    /// `gej_point_eq(P, compress(-P))` must be false.
    /// Flipping the parity bit keeps `x` and selects `(x, -y)`, the negation of `P`.
    /// The contract asserts equality, so this witness must be rejected.
    pub fn gej_point_eq_negation(ge: Ge) -> Case {
        let (parity, x) = compress(ge);

        case(GejPointEq).gej(ge_to_gej(ge)).point((parity ^ 1, x))
    }

    /// `gej_point_eq(P, (0, x))` with an off-curve `x`: must be rejected.
    pub fn gej_point_eq_off_curve_point(ge: Ge, x: Fe) -> Case {
        case(GejPointEq).gej(ge_to_gej(ge)).point((0, x))
    }

    /// Compare an off-curve `(x, y, 1)` against a compressed point with the same x and parity.
    ///
    /// The caller supplies an x from a curve point and a y off the curve at that x.
    /// Matching x and parity isolates the on-curve check.
    pub fn gej_point_eq_off_curve_gej(x: Fe, y: Fe) -> Case {
        let off_curve = (x, y);

        case(GejPointEq)
            .gej(ge_to_gej(off_curve))
            .point(compress(off_curve))
    }

    /// `safe_gej_normalize` maps any Jacobian representation of P back to P.
    pub fn safe_gej_normalize(ge: Ge, lambda: Fe) -> Case {
        case(SafeGejNormalize)
            .gej(rescale(ge, lambda))
            .expect_ge(ge)
    }
}
