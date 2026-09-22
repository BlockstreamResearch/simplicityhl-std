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
    Add128,
    Add128_64,
    FullAdd128,
    Sub128,
    FullSub128,
    Mul128,
    Mul128_64,
    CalculateNormalizerBase64,
    EstimateQuotientDigitBase64,
    DivMod128_64,
    DivMod128,
    Div128,
}

fn program() -> U128BasicMathTestProgram {
    U128BasicMathTestProgram::new(U128BasicMathTestArguments {})
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
fn add_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let result = a + b;

    case(Add128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn add_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = b - 1;

    case(Add128)
        .args(a, b)
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn add_128_64_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX / 2);
    let b = rand::thread_rng().gen_range(0..=u64::MAX) as u128;
    let result = a + b;

    case(Add128_64).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn add_128_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = rand::thread_rng().gen_range(1..=u64::MAX) as u128;
    let result = b - 1;

    case(Add128_64)
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
fn sub_128_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(0..=a);
    let result = a - b;

    case(Sub128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn sub_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(Sub128).args(a, a).expect(0).run(&context)
}

#[simplex::test]
fn sub_128_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

    let low: u64 = a as u64;
    let b = ((b_high as u128) << 64) | (low as u128);

    let carry = a < b;
    let result = a.wrapping_sub(b);

    case(Sub128)
        .args(a, b)
        .expect(result)
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_128_diff_is_u64_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_low: u64 = u64::MAX;

    let a_high = rand::thread_rng().gen_range(0..=u64::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u64::MAX);

    let a = ((a_high as u128) << 64) | (a_low as u128);
    let b = (b_high as u128) << 64; // b_low is 0

    let carry = a < b;
    let result = a.wrapping_sub(b);

    case(Sub128)
        .args(a, b)
        .expect(result)
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_128_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = u128::MAX;
    let b = 0;

    case(Sub128).args(a, b).expect(a).run(&context)
}

#[simplex::test]
fn sub_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = u128::MAX;
    let result = a + 1;

    case(Sub128)
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
fn mul_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(0..u128::MAX);
    let result = U256::from(a) * U256::from(b);

    let (result_high, result_low) = split_helper(result);

    case(Mul128)
        .args(a, b)
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_128_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..u128::MAX);
    let b = rand::thread_rng().gen_range(0..u64::MAX);
    let result = U256::from(a) * U256::from(b);

    let (result_high, result_low) = split_helper(result);

    case(Mul128_64)
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
fn div_mod_128_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
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
fn div_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = rand::thread_rng().gen_range(1..=u128::MAX);
    let result = a / b;

    case(Div128).args(a, b).expect(result).run(&context)
}

#[simplex::test]
fn div_128_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);
    let b = 0;

    case(Div128).args(a, b).expect(0).run(&context)
}

mod primitives_tests_fuzz {
    use super::*;
    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    type Builder = FuzzEngineBuilder<
        U128BasicMathTestProgram,
        U128BasicMathTestArguments,
        U128BasicMathTestWitness,
    >;
    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
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

        fn flag(mut self, expected_bool: bool) -> Self {
            self.expected_bool = expected_bool;
            self
        }

        fn module(mut self, second_expected: u128) -> Self {
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
            .module(a % b)
    }

    struct CaseFuzz {
        case: Case,
        builder: Builder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        expect: Expect,
        name: &'static str,
    }

    fn case_fuzz(function: FunctionToTest, builder: Builder, name: &'static str) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            builder,
            inputs: None,
            expect: Expect::Ok,
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
            let strategy = FuzzStrategyBuilder::<
                U128BasicMathTestArguments,
                U128BasicMathTestWitness,
                _,
            >::new()
            .with_custom_strategy(strategy)
            .build();

            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            let tx_builder = FinalTransactionBuilder::new(tx, [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, tx_builder)
                .run_with_check(FuzzExecutionCheck::new(self.name, self.expect));
            Ok(())
        }
    }

    #[simplex::fuzz]
    fn add(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (sum, carry) = a.overflowing_add(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(sum)
                    .flag(carry)
            })
        };

        case_fuzz(Add128, builder, "add").strategy(strategy).run()
    }

    #[simplex::fuzz]
    fn add_128_64(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u64()).prop_map(|(a, b)| {
                let b = b as u128;
                let (sum, carry) = a.overflowing_add(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(sum)
                    .flag(carry)
            })
        };

        case_fuzz(Add128_64, builder, "add_128_64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn full_add_no_carry(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (sum, carry) = a.overflowing_add(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(sum)
                    .flag(carry)
            })
        };

        case_fuzz(FullAdd128, builder, "full_add_no_carry")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn full_add_with_carry(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (sum, carry1) = a.overflowing_add(b);
                let (sum, carry2) = sum.overflowing_add(1);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(sum)
                    .flag(carry1 || carry2)
                    .module(1)
            })
        };

        case_fuzz(FullAdd128, builder, "full_add_with_carry")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn sub(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (diff, borrow) = a.overflowing_sub(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .flag(borrow)
            })
        };

        case_fuzz(Sub128, builder, "sub").strategy(strategy).run()
    }

    #[simplex::fuzz]
    fn sub_equal(builder: Builder) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|a| FuzzCase::first_argument(a).second_argument(a).expect(0)) };

        case_fuzz(Sub128, builder, "sub_equal")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn sub_equal_low_words(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u64()).prop_map(|(a, high)| {
                let b = ((high as u128) << 64) | (a as u64 as u128);
                let (diff, borrow) = a.overflowing_sub(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .flag(borrow)
            })
        };

        case_fuzz(Sub128, builder, "sub_equal_low_words")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn sub_max_low_word(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u64(), arb_u64()).prop_map(|(a_high, b_high)| {
                let a = ((a_high as u128) << 64) | (u64::MAX as u128);
                let b = (b_high as u128) << 64;
                let (diff, borrow) = a.overflowing_sub(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .flag(borrow)
            })
        };

        case_fuzz(Sub128, builder, "sub_max_low_word")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn full_sub_no_borrow(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (diff, borrow) = a.overflowing_sub(b);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .flag(borrow)
            })
        };

        case_fuzz(FullSub128, builder, "full_sub_no_borrow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn full_sub_with_borrow(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (diff, borrow1) = a.overflowing_sub(b);
                let (diff, borrow2) = diff.overflowing_sub(1);
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(diff)
                    .flag(borrow1 || borrow2)
                    .module(1)
            })
        };

        case_fuzz(FullSub128, builder, "full_sub_with_borrow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u128()).prop_map(|(a, b)| {
                let (high, low) = split_helper(U256::from(a) * U256::from(b));
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(high)
                    .module(low)
            })
        };

        case_fuzz(Mul128, builder, "mul").strategy(strategy).run()
    }

    #[simplex::fuzz]
    fn mul_128_64(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_u64()).prop_map(|(a, b)| {
                let b = b as u128;
                let (high, low) = split_helper(U256::from(a) * U256::from(b));
                FuzzCase::first_argument(a)
                    .second_argument(b)
                    .expect(high)
                    .module(low)
            })
        };

        case_fuzz(Mul128_64, builder, "mul_128_64")
            .strategy(strategy)
            .run()
    }

    const THRESHOLD: u128 = 1u128 << 63;

    #[simplex::fuzz]
    fn normalizer_u64_small(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (1u128..THRESHOLD).prop_map(|b| {
                FuzzCase::first_argument(0)
                    .second_argument(b)
                    .expect(THRESHOLD.div_ceil(b))
            })
        };

        case_fuzz(CalculateNormalizerBase64, builder, "normalizer_u64_small")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn normalizer_u64_large(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (THRESHOLD..=u64::MAX as u128)
                .prop_map(|b| FuzzCase::first_argument(0).second_argument(b).expect(1))
        };

        case_fuzz(CalculateNormalizerBase64, builder, "normalizer_u64_large")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn normalizer_u128(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            ((u64::MAX as u128 + 1)..=u128::MAX).prop_map(|b| {
                FuzzCase::first_argument(0)
                    .second_argument(b)
                    .expect(THRESHOLD.div_ceil(b >> 64))
                    .flag(NORMALIZER_U128_DIVISOR)
            })
        };

        case_fuzz(CalculateNormalizerBase64, builder, "normalizer_u128")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn normalizer_u64_wrong_width(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            ((u64::MAX as u128 + 1)..=u128::MAX)
                .prop_map(|b| FuzzCase::first_argument(0).second_argument(b).expect(0))
        };

        case_fuzz(
            CalculateNormalizerBase64,
            builder,
            "normalizer_u64_wrong_width",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn normalizer_u128_wrong_width(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u64_u128().prop_map(|value| {
                FuzzCase::first_argument(0)
                    .second_argument(value)
                    .expect(0)
                    .flag(NORMALIZER_U128_DIVISOR)
            })
        };

        case_fuzz(
            CalculateNormalizerBase64,
            builder,
            "normalizer_u128_wrong_width",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn normalizer_zero(builder: Builder) -> anyhow::Result<()> {
        let strategy = { Just(FuzzCase::first_argument(0).second_argument(0).expect(0)) };

        case_fuzz(CalculateNormalizerBase64, builder, "normalizer_zero")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn quotient_digit(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            ((1u64 << 63)..=u64::MAX).prop_flat_map(|b_high| {
                (Just(b_high), arb_u64(), 0..b_high, arb_u128()).prop_map(
                    |(b_high, b_low, a_high, a_low)| {
                        let a = (U256::from(a_high) << 128) | U256::from(a_low);
                        let b = ((b_high as u128) << 64) | (b_low as u128);
                        FuzzCase::first_argument(a_high as u128)
                            .second_argument(a_low)
                            .expect((a / U256::from(b)).as_u128())
                            .module(b)
                    },
                )
            })
        };

        case_fuzz(EstimateQuotientDigitBase64, builder, "quotient_digit")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn quotient_digit_too_large(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            ((1u64 << 63)..u64::MAX).prop_flat_map(|b_high| {
                (Just(b_high), arb_u64(), (b_high + 1)..=u64::MAX, arb_u128()).prop_map(
                    |(b_high, b_low, a_high, a_low)| {
                        let a = (U256::from(a_high) << 128) | U256::from(a_low);
                        let b = ((b_high as u128) << 64) | (b_low as u128);
                        FuzzCase::first_argument(a_high as u128)
                            .second_argument(a_low)
                            .expect((a / U256::from(b)).as_u128())
                            .module(b)
                    },
                )
            })
        };

        case_fuzz(
            EstimateQuotientDigitBase64,
            builder,
            "quotient_digit_too_large",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_128_64(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_non_zero_u64().prop_map(u128::from))
                .prop_map(|(a, b)| div_mod_case(a, b))
        };

        case_fuzz(DivMod128_64, builder, "div_mod_128_64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_128_64_zero(builder: Builder) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|a| FuzzCase::first_argument(a).second_argument(0).expect(0)) };

        case_fuzz(DivMod128_64, builder, "div_mod_128_64_zero")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_128(builder: Builder) -> anyhow::Result<()> {
        let strategy = { (arb_u128(), arb_non_zero_u128()).prop_map(|(a, b)| div_mod_case(a, b)) };

        case_fuzz(DivMod128, builder, "div_mod_128")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_a_less_than_b(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (0u128..u128::MAX)
                .prop_flat_map(|a| (a + 1..=u128::MAX).prop_map(move |b| div_mod_case(a, b)))
        };

        case_fuzz(DivMod128, builder, "div_mod_a_less_than_b")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_equal(builder: Builder) -> anyhow::Result<()> {
        let strategy = { arb_non_zero_u128().prop_map(|value| div_mod_case(value, value)) };

        case_fuzz(DivMod128, builder, "div_mod_equal")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_same_high_words(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u64(), arb_u64(), arb_u64()).prop_map(|(high, a_low, b_low)| {
                let a = ((high as u128) << 64) | (a_low as u128);
                let b = ((high as u128) << 64) | (b_low as u128);
                div_mod_case(a, b)
            })
        };

        case_fuzz(DivMod128, builder, "div_mod_same_high_words")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_u128(), arb_non_zero_u128())
                .prop_map(|(a, b)| FuzzCase::first_argument(a).second_argument(b).expect(a / b))
        };

        case_fuzz(Div128, builder, "div").strategy(strategy).run()
    }

    #[simplex::fuzz]
    fn div_by_zero(builder: Builder) -> anyhow::Result<()> {
        let strategy =
            { arb_u128().prop_map(|a| FuzzCase::first_argument(a).second_argument(0).expect(0)) };

        case_fuzz(Div128, builder, "div_by_zero")
            .strategy(strategy)
            .run()
    }

    // Explicit edge families complement the unrestricted arithmetic properties.
    #[simplex::fuzz]
    fn add_overflow(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u128().prop_map(|value| {
                FuzzCase::first_argument(u128::MAX)
                    .second_argument(value)
                    .expect(value - 1)
                    .flag(EXPECT_CARRY)
            })
        };

        case_fuzz(Add128, builder, "add_overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn add_128_64_overflow(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u64_u128().prop_map(|value| {
                FuzzCase::first_argument(u128::MAX)
                    .second_argument(value)
                    .expect(value - 1)
                    .flag(EXPECT_CARRY)
            })
        };

        case_fuzz(Add128_64, builder, "add_128_64_overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn full_add_overflow_with_carry(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u128().prop_map(|value| {
                FuzzCase::first_argument(u128::MAX)
                    .second_argument(value)
                    .expect(value)
                    .flag(EXPECT_CARRY)
                    .module(1)
            })
        };

        case_fuzz(FullAdd128, builder, "full_add_overflow_with_carry")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn sub_overflow(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (0u128..u128::MAX).prop_map(|a| {
                FuzzCase::first_argument(a)
                    .second_argument(u128::MAX)
                    .expect(a + 1)
                    .flag(EXPECT_BORROW)
            })
        };

        case_fuzz(Sub128, builder, "sub_overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn full_sub_overflow_with_borrow(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (0..u128::MAX).prop_map(|a| {
                FuzzCase::first_argument(a)
                    .second_argument(u128::MAX)
                    .expect(a)
                    .flag(EXPECT_BORROW)
                    .module(1)
            })
        };

        case_fuzz(FullSub128, builder, "full_sub_overflow_with_borrow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_128_div_64(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_non_zero_u64().prop_flat_map(|b| {
                (b..=u64::MAX).prop_map(move |a| div_mod_case(u128::from(a), u128::from(b)))
            })
        };

        case_fuzz(DivMod128, builder, "div_mod_128_div_64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_quotient_one(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u64(), arb_u64()).prop_flat_map(|(high, b_low)| {
                (b_low..=u64::MAX).prop_map(move |a_low| {
                    let a = (u128::from(high) << 64) | u128::from(a_low);
                    let b = (u128::from(high) << 64) | u128::from(b_low);
                    div_mod_case(a, b)
                })
            })
        };

        case_fuzz(DivMod128, builder, "div_mod_quotient_one")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_b_is_u64(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            (arb_non_zero_u64(), (u128::from(u64::MAX) + 1)..=u128::MAX)
                .prop_map(|(b, a)| div_mod_case(a, u128::from(b)))
        };

        case_fuzz(DivMod128, builder, "div_mod_b_is_u64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_b_is_u128(builder: Builder) -> anyhow::Result<()> {
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

        case_fuzz(DivMod128, builder, "div_mod_b_is_u128")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_equal_high_words_max_low_diff(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_u64().prop_map(|value| {
                let high = u128::from(value.max(1)) << 64;
                div_mod_case(high | u128::from(u64::MAX), high)
            })
        };

        case_fuzz(DivMod128, builder, "div_mod_equal_high_words_max_low_diff")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_equal_high_words_less(builder: Builder) -> anyhow::Result<()> {
        let strategy = {
            arb_u64().prop_map(|value| {
                let high = u128::from(value.max(1)) << 64;
                div_mod_case(high, high | u128::from(u64::MAX))
            })
        };

        case_fuzz(DivMod128, builder, "div_mod_equal_high_words_less")
            .strategy(strategy)
            .run()
    }
}
