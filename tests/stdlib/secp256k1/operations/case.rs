use simplicityhl_std::artifacts::tests::secp256k1::operations::OperationsProgram as Secp256k1OperationsTestProgram;
use simplicityhl_std::artifacts::tests::secp256k1::operations::derived_operations::{
    OperationsArguments as Secp256k1OperationsTestArguments,
    OperationsWitness as Secp256k1OperationsTestWitness,
};

use crate::common::case::DispatchCase;

use crate::secp256k1::math::{
    Fe, Ge, Gej, INFINITY, Point, SECP_N, SECP_P, Scalar, U256, compress, ge_negate, ge_sub,
    ge_to_gej, rescale, sub_mod,
};

/// Dispatch arms of the contract. Each index must match `is_selected(index, ..)` in the `.simf` test.
#[derive(Clone, Copy)]
enum FunctionToTest {
    GeToPoint = 0,
    PointToGej = 1,
    FeSub = 2,
    ScalarSub = 3,
    GejSub = 4,
    FeEq = 5,
    ScalarEq = 6,
    GeEq = 7,
    GejPointEq = 8,
    SafeGejNormalize = 9,
}

pub(super) fn ge_to_point(ge: Ge, expected: Point) -> Case {
    Case::new(FunctionToTest::GeToPoint)
        .ge(ge)
        .expect_point(expected)
}

pub(super) fn ge_to_point_compresses(ge: Ge) -> Case {
    ge_to_point(ge, compress(ge))
}

/// The Jacobian result is checked against a compressed point.
pub(super) fn point_to_gej(point: Point, expected: Point) -> Case {
    Case::new(FunctionToTest::PointToGej)
        .point(point)
        .expect_point(expected)
}

pub(super) fn point_to_gej_roundtrip(ge: Ge) -> Case {
    point_to_gej(compress(ge), compress(ge))
}

pub(super) fn fe_sub(a: Fe, b: Fe, expected: Fe) -> Case {
    Case::new(FunctionToTest::FeSub)
        .uints(a, b)
        .expect_uint(expected)
}

pub(super) fn fe_sub_reference(a: Fe, b: Fe) -> Case {
    fe_sub(a, b, sub_mod(a, b, SECP_P))
}

pub(super) fn scalar_sub(a: Scalar, b: Scalar, expected: Scalar) -> Case {
    Case::new(FunctionToTest::ScalarSub)
        .uints(a, b)
        .expect_uint(expected)
}

pub(super) fn scalar_sub_reference(a: Scalar, b: Scalar) -> Case {
    scalar_sub(a, b, sub_mod(a, b, SECP_N))
}

pub(super) fn gej_sub(p: Gej, q: Gej, expected: Gej) -> Case {
    Case::new(FunctionToTest::GejSub)
        .gejs(p, q)
        .expect_gej(expected)
}

pub(super) fn gej_sub_reference(p: Ge, q: Ge) -> Case {
    let expected = ge_sub(p, q).map_or(INFINITY, ge_to_gej);

    gej_sub(ge_to_gej(p), ge_to_gej(q), expected)
}

/// Accepted iff `a ≡ b (mod p)`.
pub(super) fn fe_eq(a: Fe, b: Fe) -> Case {
    Case::new(FunctionToTest::FeEq).uints(a, b)
}

/// Accepted iff `a ≡ b (mod n)`.
pub(super) fn scalar_eq(a: Scalar, b: Scalar) -> Case {
    Case::new(FunctionToTest::ScalarEq).uints(a, b)
}

/// Accepted iff both coordinates are equal mod p.
pub(super) fn ge_eq(a: Ge, b: Ge) -> Case {
    Case::new(FunctionToTest::GeEq).ges(a, b)
}

pub(super) fn ge_eq_negation(ge: Ge) -> Case {
    ge_eq(ge, ge_negate(ge))
}

/// Accepted iff `gej` is on the curve and is the point `point` encodes.
pub(super) fn gej_point_eq(gej: Gej, point: Point) -> Case {
    Case::new(FunctionToTest::GejPointEq).gej(gej).point(point)
}

pub(super) fn gej_point_eq_rescaled(ge: Ge, lambda: Fe) -> Case {
    gej_point_eq(rescale(ge, lambda), compress(ge))
}

/// Flipping the parity keeps x and encodes -P.
pub(super) fn gej_point_eq_negation(ge: Ge) -> Case {
    let (parity, x) = compress(ge);

    gej_point_eq(ge_to_gej(ge), (parity ^ 1, x))
}

pub(super) fn gej_point_eq_off_curve_point(ge: Ge, x: Fe) -> Case {
    gej_point_eq(ge_to_gej(ge), (0, x))
}

/// Matching x and parity isolates the on-curve check.
pub(super) fn gej_point_eq_off_curve_gej(x: Fe, y: Fe) -> Case {
    let off_curve = (x, y);

    gej_point_eq(ge_to_gej(off_curve), compress(off_curve))
}

pub(super) fn safe_gej_normalize(gej: Gej, expected: Ge) -> Case {
    Case::new(FunctionToTest::SafeGejNormalize)
        .gej(gej)
        .expect_ge(expected)
}

pub(super) fn safe_gej_normalize_rescaled(ge: Ge, lambda: Fe) -> Case {
    safe_gej_normalize(rescale(ge, lambda), ge)
}

#[derive(Clone, Debug)]
pub(super) struct Case {
    witness: Secp256k1OperationsTestWitness,
}

impl Case {
    fn new(function: FunctionToTest) -> Self {
        Self {
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

    fn uints(mut self, first_uint: U256, second_uint: U256) -> Self {
        self.witness.first_uint = first_uint;
        self.witness.second_uint = second_uint;
        self
    }

    fn ge(mut self, first_ge: Ge) -> Self {
        self.witness.first_ge = first_ge;
        self
    }

    fn ges(mut self, first_ge: Ge, second_ge: Ge) -> Self {
        self.witness.first_ge = first_ge;
        self.witness.second_ge = second_ge;
        self
    }

    fn gej(mut self, first_gej: Gej) -> Self {
        self.witness.first_gej = first_gej;
        self
    }

    fn gejs(mut self, first_gej: Gej, second_gej: Gej) -> Self {
        self.witness.first_gej = first_gej;
        self.witness.second_gej = second_gej;
        self
    }

    fn point(mut self, first_point: Point) -> Self {
        self.witness.first_point = first_point;
        self
    }

    fn expect_uint(mut self, expected_uint: U256) -> Self {
        self.witness.expected_uint = expected_uint;
        self
    }

    fn expect_ge(mut self, expected_ge: Ge) -> Self {
        self.witness.expected_ge = expected_ge;
        self
    }

    fn expect_gej(mut self, expected_gej: Gej) -> Self {
        self.witness.expected_gej = expected_gej;
        self
    }

    fn expect_point(mut self, expected_point: Point) -> Self {
        self.witness.expected_point = expected_point;
        self
    }
}

impl DispatchCase for Case {
    type Program = Secp256k1OperationsTestProgram;

    const PROGRAM_NAME: &'static str = "secp256k1::operations";

    fn arguments() -> Secp256k1OperationsTestArguments {
        Secp256k1OperationsTestArguments {}
    }

    fn into_witness(self) -> Secp256k1OperationsTestWitness {
        self.witness
    }
}
