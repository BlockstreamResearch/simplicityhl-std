use num_bigint::BigUint;
use secp256k1_zkp::{PublicKey, SecretKey};

use crate::common::fuzz::DispatchFuzzCase;
use crate::common::unit::Unit;
use simplicityhl_std::artifacts::tests::secp256k1::operations::OperationsProgram as Secp256k1OperationsTestProgram;
use simplicityhl_std::artifacts::tests::secp256k1::operations::derived_operations::{
    OperationsArguments as Secp256k1OperationsTestArguments,
    OperationsWitness as Secp256k1OperationsTestWitness,
};

pub const ZERO: U256 = [0; 32];
pub const ONE: U256 = secp256k1_zkp::constants::ONE;

/// The point at infinity (Z = 0). The contract's expected witness uses all-zero coordinates.
pub const INFINITY: Gej = ((ZERO, ZERO), ZERO);

// Mirror Simplicity's Fe / Scalar / Ge / Gej / Point types.

/// Big-endian 256-bit integer.
pub type U256 = [u8; 32];
/// Field element, mod p.
pub type Fe = U256;
/// Scalar, mod the group order n.
pub type Scalar = U256;
/// Affine point (x, y).
pub type Ge = (Fe, Fe);
/// Jacobian point ((X, Y), Z), representing affine (X/Z², Y/Z³).
pub type Gej = (Ge, Fe);
/// Compressed point (y parity, x).
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

pub const SECP_P: U256 = secp256k1_zkp::constants::FIELD_SIZE;
pub const SECP_N: U256 = secp256k1_zkp::constants::CURVE_ORDER;

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

/// `k·G`.
///
/// # Panics
/// If `k` is zero.
pub fn public_key(k: u128) -> PublicKey {
    let secret_key = SecretKey::from_slice(&u256(k)).expect("k must be non-zero");
    PublicKey::from_secret_key(secp256k1_zkp::SECP256K1, &secret_key)
}

/// `k·G` in affine coordinates.
pub fn g_times(k: u128) -> Ge {
    pk_to_ge(&public_key(k))
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

    pub fn unit(self) -> Unit {
        Unit {
            program: Secp256k1OperationsTestProgram::new(Case::arguments())
                .as_ref()
                .clone(),
            witness: self.witness.into(),
        }
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
            INFINITY
        } else {
            let difference = p
                .combine(&q.negate(secp256k1_zkp::SECP256K1))
                .expect("p - q non-infinity");

            ge_to_gej(pk_to_ge(&difference))
        };

        gej_sub_jacobian(ge_to_gej(pk_to_ge(&p)), ge_to_gej(pk_to_ge(&q)), expected)
    }

    /// `gej_sub(p, q) == expected` for any Jacobian operands, non-unit Z and infinity included.
    pub fn gej_sub_jacobian(p: Gej, q: Gej, expected: Gej) -> Case {
        case(GejSub).gejs(p, q).expect_gej(expected)
    }

    /// `fe_eq(a, b)`, accepted iff `a ≡ b (mod p)`.
    pub fn fe_eq(a: Fe, b: Fe) -> Case {
        case(FeEq).uints(a, b)
    }

    /// `scalar_eq(a, b)`, accepted iff `a ≡ b (mod n)`.
    pub fn scalar_eq(a: Scalar, b: Scalar) -> Case {
        case(ScalarEq).uints(a, b)
    }

    /// `ge_eq(a, b)`, accepted iff both coordinates are equal mod p.
    pub fn ge_eq(a: Ge, b: Ge) -> Case {
        case(GeEq).ges(a, b)
    }

    /// `ge_eq(P, -P)`: must be rejected, y differs.
    pub fn ge_eq_negation(ge: Ge) -> Case {
        ge_eq(ge, ge_negate(ge))
    }

    /// `gej_point_eq(gej, point)`, accepted iff `gej` is on the curve and is the point `point` encodes.
    pub fn gej_point_eq(gej: Gej, point: Point) -> Case {
        case(GejPointEq).gej(gej).point(point)
    }

    /// `gej_point_eq` holds for any Jacobian representation `(λ²X, λ³Y, λ)` of P.
    pub fn gej_point_eq_rescaled(ge: Ge, lambda: Fe) -> Case {
        gej_point_eq(rescale(ge, lambda), compress(ge))
    }

    /// `gej_point_eq(P, compress(-P))` must be false.
    /// Flipping the parity bit keeps `x` and selects `(x, -y)`, the negation of `P`.
    /// The contract asserts equality, so this witness must be rejected.
    pub fn gej_point_eq_negation(ge: Ge) -> Case {
        let (parity, x) = compress(ge);

        gej_point_eq(ge_to_gej(ge), (parity ^ 1, x))
    }

    /// `gej_point_eq(P, (0, x))` with an off-curve `x`: must be rejected.
    pub fn gej_point_eq_off_curve_point(ge: Ge, x: Fe) -> Case {
        gej_point_eq(ge_to_gej(ge), (0, x))
    }

    /// Compare an off-curve `(x, y, 1)` against a compressed point with the same x and parity.
    ///
    /// The caller supplies an x from a curve point and a y off the curve at that x.
    /// Matching x and parity isolates the on-curve check.
    pub fn gej_point_eq_off_curve_gej(x: Fe, y: Fe) -> Case {
        let off_curve = (x, y);

        gej_point_eq(ge_to_gej(off_curve), compress(off_curve))
    }

    /// `safe_gej_normalize` maps any Jacobian representation `(λ²X, λ³Y, λ)` of P back to P.
    pub fn safe_gej_normalize_rescaled(ge: Ge, lambda: Fe) -> Case {
        case(SafeGejNormalize)
            .gej(rescale(ge, lambda))
            .expect_ge(ge)
    }

    /// `safe_gej_normalize(gej) == expected`.
    pub fn safe_gej_normalize(gej: Gej, expected: Ge) -> Case {
        case(SafeGejNormalize).gej(gej).expect_ge(expected)
    }
}
