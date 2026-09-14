use primitive_types::U256;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::math::div::DivProgram as U256TestDivProgram;
use simplicityhl_std::artifacts::tests::u256::math::div::derived_div::{
    DivArguments as U256TestDivArguments, DivWitness as U256TestDivWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    CalculateNormalizerBase128,
    DivMod256_64,
    AlgorithmD256_128,
    DivMod256_128,
    DivMod256,
    Div256,
}

fn program() -> U256TestDivProgram {
    U256TestDivProgram::new(&U256TestDivArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestDivWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestDivWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            expected: None,
            second_expected: [0; 32],
        },
    }
}

impl Case {
    /// Only `first_arg`, for the arms that ignore the second operand.
    fn arg(mut self, a: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self
    }

    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = Some(expected);
        self
    }

    /// `second_expected`: the arm's second result.
    fn second(mut self, second_expected: [u8; 32]) -> Self {
        self.witness.second_expected = second_expected;
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

#[simplex::test]
fn calculate_normalizer_base_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u128 << 127;

    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let a_high = (a >> 128).as_u128();

    let norm = threshold.div_ceil(a_high);

    case(CalculateNormalizerBase128)
        .arg(a.to_big_endian())
        .expect(U256::from(norm).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_128_norm_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u128 << 127;

    // a >= 2^255 keeps a_high >= 2^127, so the divisor is already normalized
    let a = generate_u256(U256::from(2).pow(U256::from(255)), U256::MAX);
    let a_high = (a >> 128).as_u128();

    let norm = threshold.div_ceil(a_high);
    assert_eq!(norm, 1);

    case(CalculateNormalizerBase128)
        .arg(a.to_big_endian())
        .expect(U256::from(norm).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_128_norm_greater_than_1(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let threshold = 1u128 << 127;

    // a < 2^255 keeps a_high < 2^127, so the divisor has to be scaled up
    let a = generate_u256(
        U256::from(u128::MAX) + 1,
        U256::from(2).pow(U256::from(255)) - 1,
    );
    let a_high = (a >> 128).as_u128();

    let norm = threshold.div_ceil(a_high);
    assert!(norm > 1);

    case(CalculateNormalizerBase128)
        .arg(a.to_big_endian())
        .expect(U256::from(norm).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_128_a_is_u128_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let threshold = 1u128 << 127;

    let a = generate_u256(U256::one(), U256::from(threshold) - 1);

    let norm: u128 = threshold.div_ceil(a.low_u128());

    case(CalculateNormalizerBase128)
        .arg(a.to_big_endian())
        .expect(U256::from(norm).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn calculate_normalizer_base_128_b_is_zero_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let a = [0; 32];

    case(CalculateNormalizerBase128)
        .arg(a)
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn div_mod_256_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u64::MAX));

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256_64)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = [0; 32];

    case(DivMod256_64)
        .args(a.to_big_endian(), b)
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn algorithm_d_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::from(u64::MAX) + 1, U256::from(u128::MAX));

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(AlgorithmD256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn algorithm_d_256_128_fail_b_fits_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u64::MAX));

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(AlgorithmD256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn algorithm_d_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = [0; 32];

    case(AlgorithmD256_128)
        .args(a.to_big_endian(), b)
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn algorithm_d_256_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = (generate_u256(U256::one(), U256::from(u128::MAX))).to_big_endian();

    let q = U256::one().to_big_endian();
    let r = U256::zero().to_big_endian();

    case(AlgorithmD256_128)
        .args(a, a)
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::from(u64::MAX) + 1, U256::from(u128::MAX));

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_128_b_fits_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u64::MAX));

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = [0; 32];

    case(DivMod256_128)
        .args(a.to_big_endian(), b)
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn div_mod_256_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = (generate_u256(U256::one(), U256::from(u128::MAX))).to_big_endian();

    let q = U256::one().to_big_endian();
    let r = U256::zero().to_big_endian();

    case(DivMod256_128)
        .args(a, a)
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX - 1);
    let b = generate_u256(a + 1, U256::MAX);

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_div_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let a = generate_u256(b, U256::from(u128::MAX));

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
    // case where a >= b and a_high = b_high != 0
    let b_low = generate_u256(U256::zero(), U256::from(u128::MAX));
    let a_low = generate_u256(b_low, U256::from(u128::MAX));
    let high = generate_u256(U256::one(), U256::from(u128::MAX));

    let a = (high << 128) | (a_low);
    let b = (high << 128) | (b_low);

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_b_fits_into_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_b_is_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = generate_u256(U256::one(), U256::MAX - 1);
    let a = generate_u256(b + 1, U256::MAX);

    let q = (a / b).to_big_endian();
    let r = (a % b).to_big_endian();

    case(DivMod256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_a_equal_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

    case(DivMod256)
        .args(a, a)
        .expect(U256::one().to_big_endian())
        .second([0; 32])
        .run(&context)
}

#[simplex::test]
fn div_mod_256_equal_high_words_max_low_diff(context: simplex::TestContext) -> anyhow::Result<()> {
    let high = generate_u256(U256::one(), U256::from(u128::MAX));

    let a = ((high << 128) | (U256::from(u128::MAX))).to_big_endian();
    let b = (high << 128).to_big_endian();

    case(DivMod256)
        .args(a, b)
        .expect(U256::one().to_big_endian())
        .second(U256::from(u128::MAX).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn div_mod_256_eq_high_words_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let high = generate_u256(U256::one(), U256::from(u128::MAX));

    let a = (high << 128).to_big_endian();
    let b = ((high << 128) | (U256::from(u128::MAX))).to_big_endian();

    case(DivMod256)
        .args(a, b)
        .expect([0; 32])
        .second(a)
        .run(&context)
}

#[simplex::test]
fn div_mod_256_edge_case(context: simplex::TestContext) -> anyhow::Result<()> {
    let a: U256 = U256::from(2).pow(U256::from(255));
    let b = U256::from(2).pow(U256::from(127)) + U256::from(2).pow(U256::from(64)) - 1;

    let (q, r) = a.div_mod(b);

    case(DivMod256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(q.to_big_endian())
        .second(r.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn div_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::one(), U256::MAX);
    let result = (a / b).to_big_endian();

    case(Div256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn div_256_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = [0; 32];

    case(Div256)
        .args(a.to_big_endian(), b)
        .expect([0; 32])
        .run(&context)
}
