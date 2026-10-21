use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::math::primitives::PrimitivesProgram as U128BasicMathTestProgram;
use simplicityhl_std::artifacts::tests::u128::math::primitives::derived_primitives::{
    PrimitivesArguments as U128BasicMathTestArguments,
    PrimitivesWitness as U128BasicMathTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Add128Carry,
    Add128_64Carry,
    FullAdd128,
    Sub128Borrow,
    FullSub128,
    Mul128Wide,
    Mul128_64Wide,
    CalculateNormalizerBase64,
    EstimateQuotientDigitBase64,
    DivMod128_64,
    DivMod128,
}

fn program() -> U128BasicMathTestProgram {
    U128BasicMathTestProgram::new(&U128BasicMathTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128BasicMathTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128BasicMathTestWitness {
            function_index: function as u8,
            first_arg: 0,
            second_arg: 0,
            expected: None,
            expected_bool: false,
            second_expected: 0,
        },
    }
}

impl Case {
    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: u128, b: u128) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: u128) -> Self {
        self.witness.expected = Some(expected);
        self
    }

    /// `expected_bool`: the carry or borrow an arm reports beside its value.
    fn flag(mut self, flag: bool) -> Self {
        self.witness.expected_bool = flag;
        self
    }

    /// `second_expected`: the spare slot. A remainder, a low word, or a carry
    /// fed back in, depending on the arm.
    fn second(mut self, second: u128) -> Self {
        self.witness.second_expected = second;
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

fn split_helper(a: U256) -> (u128, u128) {
    let a_high = (a >> 128).as_u128();
    let a_low = a.low_u128();

    (a_high, a_low)
}

#[simplex::test]
fn add_128_carry_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b;

    case(Add128Carry).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn add_128_carry_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b - 1;

    case(Add128Carry)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn add_128_64_carry_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let result = a + b;

    case(Add128_64Carry).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn add_128_64_carry_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u64::MAX) as u128;
    let result = b - 1;

    case(Add128_64Carry)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn full_add_128_not_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b;
    let result_carry = false;
    let carry_low = 0_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn full_add_128_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b - 1;
    let result_carry = true;
    let carry_low = 0_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn full_add_128_not_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b + 1;
    let result_carry = false;
    let carry_low = 1_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn full_add_128_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b;
    let result_carry = true;
    let carry_low = 1_u128;

    case(FullAdd128)
        .args(a, b)
        .expect(result)
        .flag(result_carry)
        .second(carry_low)
        .run(&context)
}

#[simplex::test]
fn sub_128_borrow_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=a);
    let result = a - b;

    case(Sub128Borrow).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn sub_128_borrow_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(Sub128Borrow).args(a, a).expect(0).run(&context)
}

#[simplex::test]
fn sub_128_borrow_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

    let low: u64 = a as u64;
    let b = ((b_high as u128) << 64) | (low as u128);

    let carry = a < b;
    let result = a.wrapping_sub(b);

    case(Sub128Borrow)
        .args(a, b)
        .expect(result)
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_128_borrow_diff_is_u64_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_low: u64 = u64::MAX;

    let a_high = rand::thread_rng().gen_range(0..=u64::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

    let a = ((a_high as u128) << 64) | (a_low as u128);
    let b = (b_high as u128) << 64; // b_low is 0

    let carry = a < b;
    let result = a.wrapping_sub(b);

    case(Sub128Borrow)
        .args(a, b)
        .expect(result)
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_128_borrow_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = 0;

    case(Sub128Borrow).args(a, b).expect(a).run(&context)
}

#[simplex::test]
fn sub_128_borrow_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let result = a + 1;

    case(Sub128Borrow)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_borrow_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=a);
    let result = a - b;
    let result_borrow = false;
    let borrow_low = 0_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_overflow_borrow_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let result = a + 1;
    let result_borrow = true;
    let borrow_low = 0_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_borrow_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..a);
    let result = a - b - 1;
    let result_borrow = false;
    let borrow_low = 1_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn full_sub_128_overflow_borrow_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let (result, result_borrow) = a.overflowing_sub(b);

    let borrow_low = 1_u128;

    case(FullSub128)
        .args(a, b)
        .expect(result - 1)
        .flag(result_borrow)
        .second(borrow_low)
        .run(&context)
}

#[simplex::test]
fn mul_128_wide(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(0..u128::MAX);
    let result = U256::from(a) * U256::from(b);

    let (result_high, result_low) = split_helper(result);

    case(Mul128Wide)
        .args(a, b)
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_128_64_wide(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(0..u64::MAX);
    let result = U256::from(a) * U256::from(b);

    let (result_high, result_low) = split_helper(result);

    case(Mul128_64Wide)
        .args(a, b as u128)
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u128 << 63;

    let b = rand::thread_rng().gen_range(1..threshold);

    let norm: u128 = threshold.div_ceil(b);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(norm)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_big_enough_not_normalize(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let threshold = 1u128 << 63;

    let b = rand::thread_rng().gen_range(threshold..=u64::MAX as u128);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(1)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u128 << 63;

    let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);
    let b_high = b >> 64;

    let norm: u128 = threshold.div_ceil(b_high);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(norm)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u64_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range((u64::MAX as u128) + 1..=u128::MAX);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_u128_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(0)
        .flag(true)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn calculate_normalizer_base_64_b_is_zero_fail(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let b = 0;

    case(CalculateNormalizerBase64)
        .args(0, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn estimate_quotient_digit_base_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let threshold = 1u64 << 63;

    let b_high = rand::thread_rng().gen_range(threshold..=u64::MAX);
    let b_low = rand::thread_rng().gen_range(0..=u64::MAX);

    let a_high = rand::thread_rng().gen_range(0..b_high);
    let a_low = rand::thread_rng().gen_range(0..=u128::MAX);

    let a = ((U256::from(a_high)) << 128) | (U256::from(a_low));
    let b = ((b_high as u128) << 64) | (b_low as u128);

    let q = (a / b).as_u128();

    case(EstimateQuotientDigitBase64)
        .args(a_high as u128, a_low)
        .expect(q)
        .second(b)
        .run(&context)
}

#[simplex::test]
fn estimate_quotient_digit_base_64_fail(context: simplex::TestContext) -> anyhow::Result<()> {
    // expected to fail because a is to big for q to fit unto u64
    let threshold = 1u64 << 63;

    let b_high = rand::thread_rng().gen_range(threshold..u64::MAX);
    let b_low = rand::thread_rng().gen_range(0..=u64::MAX);

    let a_high = rand::thread_rng().gen_range(b_high + 1..=u64::MAX);
    let a_low = rand::thread_rng().gen_range(0..=u128::MAX);

    let a = ((U256::from(a_high)) << 128) | (U256::from(a_low));
    let b = ((b_high as u128) << 64) | (b_low as u128);

    let q = (a / b).as_u128();

    case(EstimateQuotientDigitBase64)
        .args(a_high as u128, a_low)
        .expect(q)
        .second(b)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn div_mod_128_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);

    let q = a / b;
    let r = a % b;

    case(DivMod128_64)
        .args(a, b)
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_64_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = 0;

    case(DivMod128_64)
        .args(a, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn div_mod_128_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(a + 1..=u128::MAX);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_div_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(1..=u64::MAX);
    let a = rand::thread_rng().gen_range(b..=u64::MAX) as u128;

    let q = a / b as u128;
    let r = a % b as u128;

    case(DivMod128)
        .args(a, b as u128)
        .expect(q)
        .second(r)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
    // case where a >= b and a_high = b_high != 0
    let b_low = rand::thread_rng().gen_range(0..=u64::MAX);
    let a_low = rand::thread_rng().gen_range(b_low..=u64::MAX);
    let high = rand::thread_rng().gen_range(1..u64::MAX);

    let a = ((high as u128) << 64) | (a_low as u128);
    let b = ((high as u128) << 64) | (b_low as u128);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_b_is_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let b = rand::thread_rng().gen_range(1..=u64::MAX as u128);
    let a = rand::thread_rng().gen_range(u64::MAX as u128 + 1..=u128::MAX);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_b_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let b_high = rand::thread_rng().gen_range(1..u64::MAX);
    let a_high = rand::thread_rng().gen_range(b_high + 1..=u64::MAX);

    let a = ((a_high as u128) << 64) | (rand::thread_rng().gen_range(0..u64::MAX) as u128);
    let b = ((b_high as u128) << 64) | (rand::thread_rng().gen_range(0..u64::MAX) as u128);

    let q = a / b;
    let r = a % b;

    case(DivMod128).args(a, b).expect(q).second(r).run(&context)
}

#[simplex::test]
fn div_mod_128_a_equal_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(1..=u128::MAX);

    case(DivMod128)
        .args(a, a)
        .expect(1u128)
        .second(0u128)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_equal_high_words_max_low_diff(context: simplex::TestContext) -> anyhow::Result<()> {
    let high = rand::thread_rng().gen_range(1..=u64::MAX);

    let a = ((high as u128) << 64) | (u64::MAX as u128);
    let b = (high as u128) << 64;

    case(DivMod128)
        .args(a, b)
        .expect(1u128)
        .second(u64::MAX as u128)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_eq_high_words_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let high = rand::thread_rng().gen_range(1..=u64::MAX);

    let a = (high as u128) << 64;
    let b = ((high as u128) << 64) | (u64::MAX as u128);

    case(DivMod128)
        .args(a, b)
        .expect(0u128)
        .second(a)
        .run(&context)
}

#[simplex::test]
fn div_mod_128_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = 0;

    case(DivMod128)
        .args(a, b)
        .expect(0)
        .expecting(&context, Expect::AssertFailed)
}

mod primitives_tests_fuzz {
    use crate::common::uint_fuzz::{
        run_uint_fuzz, safe_div_by_zero_strategy, safe_div_fitting_strategy,
    };

    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::Expect;
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

    const EXPECT_CARRY: bool = true;
    const EXPECT_BORROW: bool = true;
    const NORMALIZER_U128_DIVISOR: bool = true;

    fn arb_u64() -> impl Strategy<Value = u64> {
        any::<u64>()
    }

    fn arb_non_zero_u64() -> impl Strategy<Value = u64> {
        any::<u64>().prop_filter("u64 should not be zero", |index| *index != 0)
    }

    fn arb_non_zero_u64_u128() -> impl Strategy<Value = u128> {
        any::<u64>()
            .prop_filter("u64 should not be zero", |index| *index != 0)
            .prop_map(u128::from)
    }

    fn arb_u128() -> impl Strategy<Value = u128> {
        any::<u128>()
    }

    fn arb_non_zero_u128() -> impl Strategy<Value = u128> {
        arb_u128().prop_filter("u128 should not be zero", |index| *index != 0)
    }

    /// Values for one primitive operation before building the contract witness.
    #[derive(Debug, Default, Clone)]
    struct FuzzCase {
        first_arg: Option<u128>,
        second_arg: Option<u128>,
        expected: Option<u128>,
        expected_bool: bool,
        second_expected: u128,
    }

    impl FuzzCase {
        fn first_argument(first_arg: u128) -> Self {
            Self {
                first_arg: Some(first_arg),
                ..Self::default()
            }
        }

        fn second_argument(mut self, second_arg: u128) -> Self {
            self.second_arg = Some(second_arg);
            self
        }

        fn expect(mut self, expected: u128) -> Self {
            self.expected = Some(expected);
            self
        }

        fn expect_bool(mut self, expected_bool: bool) -> Self {
            self.expected_bool = expected_bool;
            self
        }

        fn second_expected(mut self, second_expected: u128) -> Self {
            self.second_expected = second_expected;
            self
        }

        fn into_witness(self, witness: U128BasicMathTestWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.expect("no first arg in witness"),
                    self.second_arg.expect("no second arg in witness"),
                )
                .expect(self.expected.expect("no expected result in witness"))
                .flag(self.expected_bool)
                .second(self.second_expected)
                .witness
                .into()
        }
    }

    fn div_mod_case(a: u128, b: u128) -> FuzzCase {
        FuzzCase::first_argument(a)
            .second_argument(b)
            .expect(a / b)
            .second_expected(a % b)
    }

    struct CaseFuzz {
        case: Case,
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        expect: Expect,
        name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        context: TestContext<FuzzMode>,
        name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            context,
            inputs: None,
            expect: Expect::Accept,
            name,
        }
    }

    impl CaseFuzz {
        fn strategy(mut self, inputs: impl Strategy<Value = FuzzCase> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
            self
        }

        fn expect(mut self, expect: Expect) -> Self {
            self.expect = expect;
            self
        }

        fn run(self) -> anyhow::Result<()> {
            let Case { witness } = self.case;
            let inputs = self.inputs.expect("a fuzz strategy must be specified");

            let strategy = inputs
                .prop_map(move |fuzz_case| {
                    let arguments: Arguments = U128BasicMathTestArguments {}.into();
                    let witness = fuzz_case.into_witness(witness.clone());

                    (arguments, witness)
                })
                .boxed();

            crate::common::fuzz::run::<U128BasicMathTestProgram>(
                &self.context,
                strategy,
                self.name,
                self.expect,
            );
            Ok(())
        }
    }

    // add_128_carry_not_overflow - accepts custom inputs for the same operation.
    // add_128_carry_overflow - accepts custom inputs for the same operation.
    // Full-width a and b; checks the wrapped sum and output carry for either outcome.
    #[simplex::fuzz]
    fn add(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (sum, carry) = a.overflowing_add(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(sum)
                    .expect_bool(carry)
            })
        };

        case_fuzz(Add128Carry, context, "add")
            .strategy(strategy)
            .run()
    }

    // add_128_64_carry_not_overflow - accepts custom inputs for the same operation.
    // add_128_64_carry_overflow - accepts custom inputs for the same operation.
    // a is u128 and b is u64; checks the wrapped sum and output carry for either outcome.
    #[simplex::fuzz]
    fn add_128_64(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u64()).prop_map(|(a, b)| {
                let b = b as u128;
                let (sum, carry) = a.overflowing_add(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(sum)
                    .expect_bool(carry)
            })
        };

        case_fuzz(Add128_64Carry, context, "add_128_64")
            .strategy(strategy)
            .run()
    }

    // full_add_128_not_overflow_carry_low_false - accepts custom inputs for the same operation.
    // full_add_128_overflow_carry_low_false - accepts custom inputs for the same operation.
    // Full-width a and b; carry_in=0; checks a+b and its output carry, including overflow.
    #[simplex::fuzz]
    fn full_add_no_carry(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (sum, carry) = a.overflowing_add(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .second_expected(0)
                    .expect(sum)
                    .expect_bool(carry)
            })
        };

        case_fuzz(FullAdd128, context, "full_add_no_carry")
            .strategy(strategy)
            .run()
    }

    // full_add_128_not_overflow_carry_low_true - accepts custom inputs for the same operation.
    // full_add_128_overflow_carry_low_true - accepts custom inputs for the same operation.
    // Full-width a and b; carry_in=1; checks wrapped a+b+1 and both carry steps.
    #[simplex::fuzz]
    fn full_add_with_carry(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (sum, carry1) = a.overflowing_add(b);
                let (sum, carry2) = sum.overflowing_add(1);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .second_expected(1)
                    .expect(sum)
                    .expect_bool(carry1 || carry2)
            })
        };

        case_fuzz(FullAdd128, context, "full_add_with_carry")
            .strategy(strategy)
            .run()
    }

    // sub_128_borrow_not_overflow - accepts custom inputs for the same operation.
    // sub_128_borrow_overflow - accepts custom inputs for the same operation.
    // Full-width a and b; checks wrapped a-b and its output borrow for either outcome.
    #[simplex::fuzz]
    fn sub(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (diff, borrow) = a.overflowing_sub(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .expect_bool(borrow)
            })
        };

        case_fuzz(Sub128Borrow, context, "sub")
            .strategy(strategy)
            .run()
    }

    // sub_128_borrow_a_eq_b - accepts custom inputs for the same operation.
    // a=b across the full width; expected result=0 and output borrow=false.
    #[simplex::fuzz]
    fn sub_equal(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|a| FuzzCase::first_argument(a).second_argument(a).expect(0)) };

        case_fuzz(Sub128Borrow, context, "sub_equal")
            .strategy(strategy)
            .run()
    }

    // sub_128_borrow_a_low_eq_b_low - accepts custom inputs for the same operation.
    // a and b share the low u64 word; high words vary independently.
    #[simplex::fuzz]
    fn sub_equal_low_words(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u64()).prop_map(|(a, high)| {
                let b = ((high as u128) << 64) | (a as u64 as u128);
                let (diff, borrow) = a.overflowing_sub(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .expect_bool(borrow)
            })
        };

        case_fuzz(Sub128Borrow, context, "sub_equal_low_words")
            .strategy(strategy)
            .run()
    }

    // sub_128_borrow_diff_is_u64_max - accepts custom inputs for the same operation.
    // a_low=u64::MAX and b_low=0; high words vary independently.
    #[simplex::fuzz]
    fn sub_max_low_word(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u64(), arb_u64()).prop_map(|(a_high, b_high)| {
                let a = ((a_high as u128) << 64) | (u64::MAX as u128);
                let b = (b_high as u128) << 64;
                let (diff, borrow) = a.overflowing_sub(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .expect_bool(borrow)
            })
        };

        case_fuzz(Sub128Borrow, context, "sub_max_low_word")
            .strategy(strategy)
            .run()
    }

    // full_sub_128_borrow_low_false - accepts custom inputs for the same operation.
    // full_sub_128_overflow_borrow_low_false - accepts custom inputs for the same operation.
    // Full-width a and b; borrow_in=0; checks a-b and its output borrow, including underflow.
    #[simplex::fuzz]
    fn full_sub_no_borrow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (diff, borrow) = a.overflowing_sub(b);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .second_expected(0)
                    .expect(diff)
                    .expect_bool(borrow)
            })
        };

        case_fuzz(FullSub128, context, "full_sub_no_borrow")
            .strategy(strategy)
            .run()
    }

    // full_sub_128_borrow_low_true - accepts custom inputs for the same operation.
    // full_sub_128_overflow_borrow_low_true - accepts custom inputs for the same operation.
    // Full-width a and b; borrow_in=1; checks wrapped a-b-1 and both borrow steps.
    #[simplex::fuzz]
    fn full_sub_with_borrow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (diff, borrow1) = a.overflowing_sub(b);
                let (diff, borrow2) = diff.overflowing_sub(1);

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .second_expected(1)
                    .expect(diff)
                    .expect_bool(borrow1 || borrow2)
            })
        };

        case_fuzz(FullSub128, context, "full_sub_with_borrow")
            .strategy(strategy)
            .run()
    }

    // mul_128_wide - accepts custom inputs for the same operation.
    // Full-width u128 operands; checks both product halves, including the high half omitted by the
    // original.
    #[simplex::fuzz]
    fn mul(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (high, low) = split_helper(U256::from(a) * U256::from(b));

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(high)
                    .second_expected(low)
            })
        };

        case_fuzz(Mul128Wide, context, "mul")
            .strategy(strategy)
            .run()
    }

    // mul_128_64_wide - accepts custom inputs for the same operation.
    // a is u128 and b is u64; checks both halves of the widened product.
    #[simplex::fuzz]
    fn mul_128_64(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u64()).prop_map(|(a, b)| {
                let b = b as u128;
                let (high, low) = split_helper(U256::from(a) * U256::from(b));

                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(high)
                    .second_expected(low)
            })
        };

        case_fuzz(Mul128_64Wide, context, "mul_128_64")
            .strategy(strategy)
            .run()
    }

    const THRESHOLD: u128 = 1u128 << 63;

    // calculate_normalizer_base_64_b_is_u64 - accepts custom inputs for the same operation.
    // 1<=b<2^63; selects the low word; expected normalizer=ceil(2^63/b).
    #[simplex::fuzz]
    fn normalizer_u64_small(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (1u128..THRESHOLD).prop_map(|b| {
                FuzzCase::first_argument(0)
                    .second_argument(b)
                    .expect(THRESHOLD.div_ceil(b))
            })
        };

        case_fuzz(CalculateNormalizerBase64, context, "normalizer_u64_small")
            .strategy(strategy)
            .run()
    }

    // calculate_normalizer_base_64_b_is_big_enough_not_normalize - accepts custom inputs for the
    // same operation.
    // 2^63<=b<=u64::MAX; selects the low word; expected normalizer=1.
    #[simplex::fuzz]
    fn normalizer_u64_large(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (THRESHOLD..=u64::MAX as u128)
                .prop_map(|b| FuzzCase::first_argument(0).second_argument(b).expect(1))
        };

        case_fuzz(CalculateNormalizerBase64, context, "normalizer_u64_large")
            .strategy(strategy)
            .run()
    }

    // calculate_normalizer_base_64_b_is_u128 - accepts custom inputs for the same operation.
    // u64::MAX<b<=u128::MAX; selects the high word; expects ceil(2^63/b_high).
    #[simplex::fuzz]
    fn normalizer_u128(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            ((u64::MAX as u128 + 1)..=u128::MAX).prop_map(|b| {
                FuzzCase::first_argument(0)
                    .second_argument(b)
                    .expect(THRESHOLD.div_ceil(b >> 64))
                    .expect_bool(NORMALIZER_U128_DIVISOR)
            })
        };

        case_fuzz(CalculateNormalizerBase64, context, "normalizer_u128")
            .strategy(strategy)
            .run()
    }

    // calculate_normalizer_base_64_b_is_u64_fail - accepts custom inputs for the same operation.
    // b>u64::MAX with the low-word selector; expects an assertion failure.
    #[simplex::fuzz]
    fn normalizer_u64_wrong_width(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            ((u64::MAX as u128 + 1)..=u128::MAX)
                .prop_map(|b| FuzzCase::first_argument(0).second_argument(b).expect(0))
        };

        case_fuzz(
            CalculateNormalizerBase64,
            context,
            "normalizer_u64_wrong_width",
        )
        .strategy(strategy)
        .expect(Expect::Reject)
        .run()
    }

    // calculate_normalizer_base_64_b_is_u128_fail - accepts custom inputs for the same operation.
    // 1<=b<=u64::MAX with the high-word selector; expects an assertion failure.
    #[simplex::fuzz]
    fn normalizer_u128_wrong_width(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u64_u128().prop_map(|value| {
                FuzzCase::first_argument(0)
                    .second_argument(value)
                    .expect(0)
                    .expect_bool(NORMALIZER_U128_DIVISOR)
            })
        };

        case_fuzz(
            CalculateNormalizerBase64,
            context,
            "normalizer_u128_wrong_width",
        )
        .strategy(strategy)
        .expect(Expect::Reject)
        .run()
    }

    // calculate_normalizer_base_64_b_is_zero_fail - accepts custom inputs for the same operation.
    // b=0 is fixed with Just; expects an assertion failure.
    #[simplex::fuzz]
    fn normalizer_zero(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = { Just(FuzzCase::first_argument(0).second_argument(0).expect(0)) };

        case_fuzz(CalculateNormalizerBase64, context, "normalizer_zero")
            .strategy(strategy)
            .expect(Expect::Reject)
            .run()
    }

    // estimate_quotient_digit_base_64 - accepts custom inputs for the same operation.
    // 2^63<=b_high<=u64::MAX and a_high<b_high; the reference quotient fits u64.
    #[simplex::fuzz]
    fn quotient_digit(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            ((1u64 << 63)..=u64::MAX).prop_flat_map(|b_high| {
                (Just(b_high), arb_u64(), 0..b_high, arb_u128()).prop_map(
                    |(b_high, b_low, a_high, a_low)| {
                        let a = (U256::from(a_high) << 128) | U256::from(a_low);
                        let b = ((b_high as u128) << 64) | (b_low as u128);
                        FuzzCase::first_argument(a_high as u128)
                            .second_argument(a_low)
                            .expect((a / U256::from(b)).as_u128())
                            .second_expected(b)
                    },
                )
            })
        };

        case_fuzz(EstimateQuotientDigitBase64, context, "quotient_digit")
            .strategy(strategy)
            .run()
    }

    // estimate_quotient_digit_base_64_fail - accepts custom inputs for the same operation.
    // 2^63<=b_high<u64::MAX and a_high>b_high; expects the original assertion-failure outcome.
    #[simplex::fuzz]
    fn quotient_digit_too_large(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            ((1u64 << 63)..u64::MAX).prop_flat_map(|b_high| {
                (Just(b_high), arb_u64(), (b_high + 1)..=u64::MAX, arb_u128()).prop_map(
                    |(b_high, b_low, a_high, a_low)| {
                        let a = (U256::from(a_high) << 128) | U256::from(a_low);
                        let b = ((b_high as u128) << 64) | (b_low as u128);
                        FuzzCase::first_argument(a_high as u128)
                            .second_argument(a_low)
                            .expect((a / U256::from(b)).as_u128())
                            .second_expected(b)
                    },
                )
            })
        };

        case_fuzz(
            EstimateQuotientDigitBase64,
            context,
            "quotient_digit_too_large",
        )
        .strategy(strategy)
        .expect(Expect::Reject)
        .run()
    }

    // div_mod_128_64 - accepts custom inputs for the same operation.
    // a is full-width u128; 1<=b<=u64::MAX; checks quotient and remainder.
    #[simplex::fuzz]
    fn div_mod_128_64(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_non_zero_u64().prop_map(u128::from))
                .prop_map(|(a, b)| div_mod_case(a, b))
        };

        case_fuzz(DivMod128_64, context, "div_mod_128_64")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_64_div_by_zero - accepts custom inputs for the same operation.
    // a is full-width u128 and b=0; expects an assertion failure.
    #[simplex::fuzz]
    fn div_mod_128_64_zero(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|a| FuzzCase::first_argument(a).second_argument(0).expect(0)) };

        case_fuzz(DivMod128_64, context, "div_mod_128_64_zero")
            .strategy(strategy)
            .expect(Expect::Reject)
            .run()
    }

    // div_mod_128_a_less_than_b - accepts custom inputs for the same operation.
    // div_mod_128_b_is_u64 - accepts custom inputs for the same operation.
    // div_mod_128_b_is_u128 - accepts custom inputs for the same operation.
    // div_mod_128_a_equal_b - accepts custom inputs for the same operation.
    // a is full-width u128 and b is nonzero u128; checks quotient and remainder across branches.
    #[simplex::fuzz]
    fn div_mod_128(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = { (arb_u128(), arb_non_zero_u128()).prop_map(|(a, b)| div_mod_case(a, b)) };

        case_fuzz(DivMod128, context, "div_mod_128")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_a_less_than_b - accepts custom inputs for the same operation.
    // 0<=a<u128::MAX and a<b<=u128::MAX; expects q=0 and r=a.
    #[simplex::fuzz]
    fn div_mod_a_less_than_b(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (0u128..u128::MAX)
                .prop_flat_map(|a| (a + 1..=u128::MAX).prop_map(move |b| div_mod_case(a, b)))
        };

        case_fuzz(DivMod128, context, "div_mod_a_less_than_b")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_a_equal_b - accepts custom inputs for the same operation.
    // a=b>0; expects q=1 and r=0.
    #[simplex::fuzz]
    fn div_mod_equal(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = { arb_non_zero_u128().prop_map(|value| div_mod_case(value, value)) };

        case_fuzz(DivMod128, context, "div_mod_equal")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_q_is_1 - accepts custom inputs for the same operation.
    // div_mod_128_eq_high_words_a_less_than_b - accepts custom inputs for the same operation.
    // a_high=b_high>0; low words are independent, covering q=0 or q=1.
    #[simplex::fuzz]
    fn div_mod_same_high_words(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u64(), arb_u64(), arb_u64()).prop_map(|(high, a_low, b_low)| {
                let a = ((high as u128) << 64) | (a_low as u128);
                let b = ((high as u128) << 64) | (b_low as u128);

                div_mod_case(a, b)
            })
        };

        case_fuzz(DivMod128, context, "div_mod_same_high_words")
            .strategy(strategy)
            .run()
    }

    // No direct simplex::test counterpart - additional general property.
    // Additional API division property, not a direct primitive-dispatch counterpart; a is u128 and
    // b>0.
    #[simplex::fuzz]
    fn div(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        run_uint_fuzz::<u128>(context, safe_div_fitting_strategy::<u128>(), Expect::Accept)
    }

    // div_mod_128_div_by_zero - accepts custom inputs for the same operation.
    // Uses API div_128, which calls div_mod_128 indirectly; a is u128, b=0, and assertion failure
    // is expected.
    #[simplex::fuzz]
    fn div_by_zero(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        run_uint_fuzz::<u128>(context, safe_div_by_zero_strategy::<u128>(), Expect::Reject)
    }

    // Explicit edge families complement the unrestricted arithmetic properties.
    // add_128_carry_overflow - accepts custom inputs for the same operation.
    // a=u128::MAX and b>0; wrapped result=b-1 and output carry=true.
    #[simplex::fuzz]
    fn add_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u128().prop_map(|value| {
                FuzzCase::first_argument(u128::MAX)
                    .second_argument(value)
                    .expect(value - 1)
                    .expect_bool(EXPECT_CARRY)
            })
        };

        case_fuzz(Add128Carry, context, "add_overflow")
            .strategy(strategy)
            .run()
    }

    // add_128_64_carry_overflow - accepts custom inputs for the same operation.
    // a=u128::MAX and 1<=b<=u64::MAX; result=b-1 and output carry=true.
    #[simplex::fuzz]
    fn add_128_64_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u64_u128().prop_map(|value| {
                FuzzCase::first_argument(u128::MAX)
                    .second_argument(value)
                    .expect(value - 1)
                    .expect_bool(EXPECT_CARRY)
            })
        };

        case_fuzz(Add128_64Carry, context, "add_128_64_overflow")
            .strategy(strategy)
            .run()
    }

    // full_add_128_overflow_carry_low_true - accepts custom inputs for the same operation.
    // a=u128::MAX, b>0, carry_in=1; wrapped result=b and output carry=true.
    #[simplex::fuzz]
    fn full_add_overflow_with_carry(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u128().prop_map(|value| {
                FuzzCase::first_argument(u128::MAX)
                    .second_argument(value)
                    .second_expected(1)
                    .expect(value)
                    .expect_bool(EXPECT_CARRY)
            })
        };

        case_fuzz(FullAdd128, context, "full_add_overflow_with_carry")
            .strategy(strategy)
            .run()
    }

    // sub_128_borrow_overflow - accepts custom inputs for the same operation.
    // 0<=a<u128::MAX and b=u128::MAX; wrapped result=a+1 and output borrow=true.
    #[simplex::fuzz]
    fn sub_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (0u128..u128::MAX).prop_map(|a| {
                FuzzCase::first_argument(a)
                    .second_argument(u128::MAX)
                    .expect(a + 1)
                    .expect_bool(EXPECT_BORROW)
            })
        };

        case_fuzz(Sub128Borrow, context, "sub_overflow")
            .strategy(strategy)
            .run()
    }

    // full_sub_128_overflow_borrow_low_true - accepts custom inputs for the same operation.
    // 0<=a<u128::MAX, b=u128::MAX, borrow_in=1; result=a and output borrow=true.
    #[simplex::fuzz]
    fn full_sub_overflow_with_borrow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (0..u128::MAX).prop_map(|a| {
                FuzzCase::first_argument(a)
                    .second_argument(u128::MAX)
                    .second_expected(1)
                    .expect(a)
                    .expect_bool(EXPECT_BORROW)
            })
        };

        case_fuzz(FullSub128, context, "full_sub_overflow_with_borrow")
            .strategy(strategy)
            .run()
    }

    // No direct simplex::test counterpart - new input-bit boundary.
    // New boundary: a=u128::MAX, b=0, carry_in=1; only the input carry causes result=0 and
    // carry_out=true.
    #[simplex::fuzz]
    fn full_add_carry_in_causes_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        // MAX + 0 fits; only the input carry causes overflow.
        let strategy = Just(u128::MAX).prop_map(|a| {
            FuzzCase::first_argument(a)
                .second_argument(0)
                .second_expected(1)
                .expect(0)
                .expect_bool(true)
        });

        case_fuzz(FullAdd128, context, "full_add_carry_in_causes_overflow")
            .strategy(strategy)
            .run()
    }

    // No direct simplex::test counterpart - new input-bit boundary.
    // New boundary: a=b across the full width, borrow_in=1; only the input borrow causes result=MAX
    // and borrow_out=true.
    #[simplex::fuzz]
    fn full_sub_borrow_in_causes_underflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        // a - a is zero; only the input borrow causes underflow.
        let strategy = arb_u128().prop_map(|a| {
            FuzzCase::first_argument(a)
                .second_argument(a)
                .second_expected(1)
                .expect(u128::MAX)
                .expect_bool(true)
        });

        case_fuzz(FullSub128, context, "full_sub_borrow_in_causes_underflow")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_div_64 - accepts custom inputs for the same operation.
    // 1<=b<=a<=u64::MAX; checks the small-operands branch of div_mod_128.
    #[simplex::fuzz]
    fn div_mod_128_div_64(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u64().prop_flat_map(|b| {
                (b..=u64::MAX).prop_map(move |a| div_mod_case(u128::from(a), u128::from(b)))
            })
        };

        case_fuzz(DivMod128, context, "div_mod_128_div_64")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_q_is_1 - accepts custom inputs for the same operation.
    // a_high=b_high>0 and a_low>=b_low; expects q=1 and the exact remainder.
    #[simplex::fuzz]
    fn div_mod_quotient_one(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u64(), arb_u64()).prop_flat_map(|(high, b_low)| {
                (b_low..=u64::MAX).prop_map(move |a_low| {
                    let a = (u128::from(high) << 64) | u128::from(a_low);
                    let b = (u128::from(high) << 64) | u128::from(b_low);
                    div_mod_case(a, b)
                })
            })
        };

        case_fuzz(DivMod128, context, "div_mod_quotient_one")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_b_is_u64 - accepts custom inputs for the same operation.
    // a>u64::MAX and 1<=b<=u64::MAX; checks the small-divisor branch.
    #[simplex::fuzz]
    fn div_mod_b_is_u64(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u64(), (u128::from(u64::MAX) + 1)..=u128::MAX)
                .prop_map(|(b, a)| div_mod_case(a, u128::from(b)))
        };

        case_fuzz(DivMod128, context, "div_mod_b_is_u64")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_b_is_u128 - accepts custom inputs for the same operation.
    // a_high>b_high>0 and both low words are nonzero; checks the wide-divisor branch.
    #[simplex::fuzz]
    fn div_mod_b_is_u128(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            (1u64..u64::MAX).prop_flat_map(|b_high| {
                (
                    (b_high + 1)..=u64::MAX,
                    arb_non_zero_u64(),
                    arb_non_zero_u64(),
                )
                    .prop_map(move |(a_high, a_low, b_low)| {
                        let a = (u128::from(a_high) << 64) | u128::from(a_low);
                        let b = (u128::from(b_high) << 64) | u128::from(b_low);

                        div_mod_case(a, b)
                    })
            })
        };

        case_fuzz(DivMod128, context, "div_mod_b_is_u128")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_equal_high_words_max_low_diff - accepts custom inputs for the same operation.
    // a_high=b_high>0, a_low=u64::MAX, b_low=0; expects q=1 and r=u64::MAX.
    #[simplex::fuzz]
    fn div_mod_equal_high_words_max_low_diff(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_u64().prop_map(|value| {
                let high = u128::from(value.max(1)) << 64;

                div_mod_case(high | u128::from(u64::MAX), high)
            })
        };

        case_fuzz(DivMod128, context, "div_mod_equal_high_words_max_low_diff")
            .strategy(strategy)
            .run()
    }

    // div_mod_128_eq_high_words_a_less_than_b - accepts custom inputs for the same operation.
    // a_high=b_high>0, a_low=0, b_low=u64::MAX; expects q=0 and r=a.
    #[simplex::fuzz]
    fn div_mod_equal_high_words_less(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            arb_u64().prop_map(|value| {
                let high = u128::from(value.max(1)) << 64;

                div_mod_case(high, high | u128::from(u64::MAX))
            })
        };

        case_fuzz(DivMod128, context, "div_mod_equal_high_words_less")
            .strategy(strategy)
            .run()
    }
}
