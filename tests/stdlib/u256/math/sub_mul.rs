use primitive_types::U256;
use primitive_types::U512;
use rand::Rng;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::math::sub_mul::SubMulProgram as U256TestSubMulProgram;
use simplicityhl_std::artifacts::tests::u256::math::sub_mul::derived_sub_mul::{
    SubMulArguments as U256TestSubMulArguments, SubMulWitness as U256TestSubMulWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Sub256,
    Mul256,
    Mul256_64,
    Mul256_128,
    Mul512_128,
    SafeMul256_128,
}

fn program() -> U256TestSubMulProgram {
    U256TestSubMulProgram::new(U256TestSubMulArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestSubMulWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestSubMulWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            third_arg: 0,
            expected: None,
            expected_bool: false,
            second_expected: [0; 32],
            third_expected: [0; 32],
        },
    }
}

impl Case {
    /// The two operands, `first_arg` and `second_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self
    }

    /// `third_arg`: the extra operand only some arms take.
    fn third_arg(mut self, c: u128) -> Self {
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = Some(expected);
        self
    }

    /// `expected_bool`: the boolean the arm should report.
    fn flag(mut self, expected_bool: bool) -> Self {
        self.witness.expected_bool = expected_bool;
        self
    }

    /// `second_expected`: the arm's second result.
    fn second(mut self, second_expected: [u8; 32]) -> Self {
        self.witness.second_expected = second_expected;
        self
    }

    /// `third_expected`: the arm's third result.
    fn third(mut self, third_expected: [u8; 32]) -> Self {
        self.witness.third_expected = third_expected;
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

fn split_u512(a: [u8; 64]) -> ([u8; 32], [u8; 32]) {
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    (high.to_big_endian(), low.to_big_endian())
}

#[simplex::test]
fn sub_256_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b = generate_u256(U256::zero(), a);
    let result = (a - b).to_big_endian();

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn sub_256_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX).to_big_endian();

    case(Sub256).args(a, a).expect([0; 32]).run(&context)
}

#[simplex::test]
fn sub_256_a_low_eq_b_low(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u128::MAX);

    let low: u128 = a.low_u128();
    let b = (U256::from(b_high) << 128) | U256::from(low);

    let (result, carry) = a.overflowing_sub(b);

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result.to_big_endian())
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_256_diff_is_u128_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_low: u128 = u128::MAX;

    let a_high = rand::thread_rng().gen_range(0..=u128::MAX);
    let b_high = rand::thread_rng().gen_range(0..=u128::MAX);

    let a = (U256::from(a_high) << 128) | U256::from(a_low);
    let b = (U256::from(b_high)) << 128; // b_low is 0

    let (result, carry) = a.overflowing_sub(b);

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result.to_big_endian())
        .flag(carry)
        .run(&context)
}

#[simplex::test]
fn sub_256_diff_is_u256_max(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX.to_big_endian();
    let b = U256::zero();

    case(Sub256)
        .args(a, b.to_big_endian())
        .expect(a)
        .run(&context)
}

#[simplex::test]
fn sub_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX - 1);
    let b = U256::MAX;
    let result = a + 1;

    case(Sub256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result.to_big_endian())
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn mul_256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::MAX);
    let result = a.full_mul(b).to_big_endian();

    let (result_high, result_low) = split_u512(result);

    case(Mul256)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_256_64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u64::MAX));
    let result = a.full_mul(b).to_big_endian();

    let (result_high, result_low) = split_u512(result);

    case(Mul256_64)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let result = a.full_mul(b).to_big_endian();

    let (result_high, result_low) = split_u512(result);

    case(Mul256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result_high)
        .second(result_low)
        .run(&context)
}

#[simplex::test]
fn mul_512_128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a_1 = generate_u256(U256::one(), U256::MAX);
    let a_0 = generate_u256(U256::one(), U256::MAX);
    let b = rand::thread_rng().gen_range(1..=u128::MAX);

    let result_low = U512::from(a_0) * U512::from(b);
    let result_high = U512::from(a_1) * U512::from(b);

    let (res_1, res_0) = split_u512(result_low.to_big_endian());
    let (res_3, res_2) = split_u512(result_high.to_big_endian());

    let res_2_1 = U512::from_big_endian(&res_1) + U512::from_big_endian(&res_2);
    let (res_3_1, res_2_1) = split_u512(res_2_1.to_big_endian());

    let res_3_final =
        (U256::from_big_endian(&res_3_1) + U256::from_big_endian(&res_3)).to_big_endian();

    case(Mul512_128)
        .args(a_1.to_big_endian(), a_0.to_big_endian())
        .third_arg(b)
        .expect(res_3_final)
        .second(res_2_1)
        .third(res_0)
        .run(&context)
}

#[simplex::test]
fn safe_mul_256_128_fitting(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u128::MAX));
    let b = generate_u256(U256::zero(), U256::from(u128::MAX));

    case(SafeMul256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect((a * b).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_mul_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let b = U256::from(u128::MAX);

    case(SafeMul256_128)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

mod sub_mul_tests_fuzz {
    use super::*;
    use std::cmp::max;

    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    const CARRY_TRUE: bool = true;

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type U256SubMulFuzzEngineBuilder =
        FuzzEngineBuilder<U256TestSubMulProgram, U256TestSubMulArguments, U256TestSubMulWitness>;

    fn arb_u64() -> impl Strategy<Value = u64> {
        any::<u64>()
    }

    fn arb_non_zero_u64() -> impl Strategy<Value = u64> {
        arb_u64().prop_map(|value| max(value, 1))
    }

    fn arb_non_zero_64bit_u256() -> impl Strategy<Value = U256> {
        arb_non_zero_u64().prop_map(U256::from)
    }

    fn arb_u128() -> impl Strategy<Value = u128> {
        any::<u128>()
    }

    fn arb_non_zero_u128() -> impl Strategy<Value = u128> {
        arb_u128().prop_map(|value| max(value, 1))
    }

    fn arb_128bit_u256() -> impl Strategy<Value = U256> {
        arb_u128().prop_map(U256::from)
    }

    fn arb_non_zero_128bit_u256() -> impl Strategy<Value = U256> {
        arb_non_zero_u128().prop_map(U256::from)
    }

    fn arb_u256() -> impl Strategy<Value = U256> {
        any::<[u8; 32]>().prop_map(|bytes| U256::from_big_endian(&bytes))
    }

    fn arb_non_zero_u256() -> impl Strategy<Value = U256> {
        arb_u256().prop_map(|value| max(value, U256::one()))
    }

    fn arb_u256_in_range(low: U256, high: U256) -> impl Strategy<Value = U256> {
        assert!(low <= high);

        let range = high - low;

        arb_u256().prop_map(move |value| {
            if range == U256::MAX {
                value
            } else {
                low + value % (range + U256::one())
            }
        })
    }

    #[derive(Debug)]
    struct FuzzCase {
        first_arg: U256,
        second_arg: U256,
        third_arg: u128,
        expected: U256,
        expected_bool: bool,
        second_expected: U256,
        third_expected: U256,
    }

    impl FuzzCase {
        fn new(first_arg: U256, second_arg: U256) -> Self {
            Self {
                first_arg,
                second_arg,
                third_arg: 0,
                expected: U256::zero(),
                expected_bool: false,
                second_expected: U256::zero(),
                third_expected: U256::zero(),
            }
        }

        fn third_arg(mut self, third_arg: u128) -> Self {
            self.third_arg = third_arg;
            self
        }

        fn expect(mut self, expected: U256) -> Self {
            self.expected = expected;
            self
        }

        fn flag(mut self, expected_bool: bool) -> Self {
            self.expected_bool = expected_bool;
            self
        }

        fn second(mut self, second_expected: U256) -> Self {
            self.second_expected = second_expected;
            self
        }

        fn third(mut self, third_expected: U256) -> Self {
            self.third_expected = third_expected;
            self
        }

        fn into_witness(self, witness: U256TestSubMulWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.to_big_endian(),
                    self.second_arg.to_big_endian(),
                )
                .third_arg(self.third_arg)
                .expect(self.expected.to_big_endian())
                .flag(self.expected_bool)
                .second(self.second_expected.to_big_endian())
                .third(self.third_expected.to_big_endian())
                .witness
                .into()
        }
    }

    struct CaseFuzz {
        case: Case,
        builder: U256SubMulFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U256SubMulFuzzEngineBuilder,
        test_name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            builder,
            inputs: None,
            test_name,
            expect: Expect::Ok,
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

        fn build_initial_tx() -> FinalTransaction {
            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            tx
        }

        fn run(self) -> anyhow::Result<()> {
            let Case { witness } = self.case;
            let inputs = self.inputs.expect("a fuzz strategy must be specified");

            let strategy = inputs
                .prop_map(move |case| {
                    let arguments: Arguments = U256TestSubMulArguments {}.into();
                    let witness = case.into_witness(witness.clone());

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U256TestSubMulArguments, U256TestSubMulWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();

            let transaction_builder =
                FinalTransactionBuilder::new(Self::build_initial_tx(), [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, self.expect));

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn sub_256_not_overflow(
        fuzz_engine_builder: U256SubMulFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();

            a.prop_flat_map(|a| {
                arb_u256_in_range(U256::zero(), a).prop_map(move |b| {
                    let result = a - b;

                    FuzzCase::new(a, b).expect(result)
                })
            })
        };

        case_fuzz(
            Sub256,
            fuzz_engine_builder,
            "u256 subtraction without overflow",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn sub_256_a_eq_b(fuzz_engine_builder: U256SubMulFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(|a| FuzzCase::new(a, a).expect(U256::zero()));

        case_fuzz(
            Sub256,
            fuzz_engine_builder,
            "u256 subtraction equal operands",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn sub_256_a_low_eq_b_low(
        fuzz_engine_builder: U256SubMulFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b_high = arb_u128();

            (a, b_high).prop_map(|(a, b_high)| {
                let low = U256::from(a.low_u128());
                let b = (U256::from(b_high) << 128) | low;
                let (result, carry) = a.overflowing_sub(b);

                FuzzCase::new(a, b).expect(result).flag(carry)
            })
        };

        case_fuzz(
            Sub256,
            fuzz_engine_builder,
            "u256 subtraction equal low halves",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn sub_256_diff_is_u128_max(
        fuzz_engine_builder: U256SubMulFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a_high = arb_u128();
            let b_high = arb_u128();

            (a_high, b_high).prop_map(|(a_high, b_high)| {
                let a = (U256::from(a_high) << 128) | U256::from(u128::MAX);
                let b = U256::from(b_high) << 128;
                let (result, carry) = a.overflowing_sub(b);

                FuzzCase::new(a, b).expect(result).flag(carry)
            })
        };

        case_fuzz(
            Sub256,
            fuzz_engine_builder,
            "u256 subtraction u128 max difference",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn sub_256_overflow(fuzz_engine_builder: U256SubMulFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::one(), U256::MAX - 1);
            let b = U256::MAX;

            a.prop_map(move |a| {
                let result = a + U256::one();

                FuzzCase::new(a, b).expect(result).flag(CARRY_TRUE)
            })
        };

        case_fuzz(Sub256, fuzz_engine_builder, "u256 subtraction overflow")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_256(fuzz_engine_builder: U256SubMulFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();
            let b = arb_non_zero_u256();

            (a, b).prop_map(|(a, b)| {
                let (high, low) = split_u512(a.full_mul(b).to_big_endian());
                let high = U256::from_big_endian(&high);
                let low = U256::from_big_endian(&low);

                FuzzCase::new(a, b).expect(high).second(low)
            })
        };

        case_fuzz(Mul256, fuzz_engine_builder, "u256 multiplication")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_256_64(fuzz_engine_builder: U256SubMulFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();
            let b = arb_non_zero_64bit_u256();

            (a, b).prop_map(|(a, b)| {
                let (high, low) = split_u512(a.full_mul(b).to_big_endian());
                let high = U256::from_big_endian(&high);
                let low = U256::from_big_endian(&low);

                FuzzCase::new(a, b).expect(high).second(low)
            })
        };

        case_fuzz(Mul256_64, fuzz_engine_builder, "u256 by u64 multiplication")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_256_128(fuzz_engine_builder: U256SubMulFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();
            let b = arb_non_zero_128bit_u256();

            (a, b).prop_map(|(a, b)| {
                let (high, low) = split_u512(a.full_mul(b).to_big_endian());
                let high = U256::from_big_endian(&high);
                let low = U256::from_big_endian(&low);

                FuzzCase::new(a, b).expect(high).second(low)
            })
        };

        case_fuzz(
            Mul256_128,
            fuzz_engine_builder,
            "u256 by u128 multiplication",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn mul_512_128(fuzz_engine_builder: U256SubMulFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a_1 = arb_non_zero_u256();
            let a_0 = arb_non_zero_u256();
            let b = 1..=u128::MAX;

            (a_1, a_0, b).prop_map(|(a_1, a_0, b)| {
                let result_low = U512::from(a_0) * U512::from(b);
                let result_high = U512::from(a_1) * U512::from(b);

                let (res_1, res_0) = split_u512(result_low.to_big_endian());
                let (res_3, res_2) = split_u512(result_high.to_big_endian());

                let res_2_1 = U512::from_big_endian(&res_1) + U512::from_big_endian(&res_2);
                let (res_3_1, res_2_1) = split_u512(res_2_1.to_big_endian());
                let res_3_final = U256::from_big_endian(&res_3_1) + U256::from_big_endian(&res_3);
                let res_2_1 = U256::from_big_endian(&res_2_1);
                let res_0 = U256::from_big_endian(&res_0);

                FuzzCase::new(a_1, a_0)
                    .third_arg(b)
                    .expect(res_3_final)
                    .second(res_2_1)
                    .third(res_0)
            })
        };

        case_fuzz(
            Mul512_128,
            fuzz_engine_builder,
            "u512 by u128 multiplication",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn safe_mul_256_128_fitting(
        fuzz_engine_builder: U256SubMulFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_128bit_u256();
            let b = arb_128bit_u256();

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a * b))
        };

        case_fuzz(
            SafeMul256_128,
            fuzz_engine_builder,
            "safe u256 by u128 multiplication fitting",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn safe_mul_256_128_overflow(
        fuzz_engine_builder: U256SubMulFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let b = U256::from(u128::MAX);
        let first_overflowing_a = U256::MAX / b + U256::one();
        let strategy = arb_u256_in_range(first_overflowing_a, U256::MAX)
            .prop_map(move |a| FuzzCase::new(a, b));

        case_fuzz(
            SafeMul256_128,
            fuzz_engine_builder,
            "safe u256 by u128 multiplication overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }
}
