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
    U256TestDivProgram::new(U256TestDivArguments {})
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

mod div_tests_fuzz {
    use super::*;
    use std::cmp::max;

    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type U256DivFuzzEngineBuilder =
        FuzzEngineBuilder<U256TestDivProgram, U256TestDivArguments, U256TestDivWitness>;

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
        expected: U256,
        second_expected: U256,
    }

    impl FuzzCase {
        fn arg(first_arg: U256) -> Self {
            Self::new(first_arg, U256::zero())
        }

        fn new(first_arg: U256, second_arg: U256) -> Self {
            Self {
                first_arg,
                second_arg,
                expected: U256::zero(),
                second_expected: U256::zero(),
            }
        }

        fn expect(mut self, expected: U256) -> Self {
            self.expected = expected;
            self
        }

        fn second(mut self, second_expected: U256) -> Self {
            self.second_expected = second_expected;
            self
        }

        fn into_witness(self, witness: U256TestDivWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.to_big_endian(),
                    self.second_arg.to_big_endian(),
                )
                .expect(self.expected.to_big_endian())
                .second(self.second_expected.to_big_endian())
                .witness
                .into()
        }
    }

    struct CaseFuzz {
        case: Case,
        builder: U256DivFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U256DivFuzzEngineBuilder,
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
                    let arguments: Arguments = U256TestDivArguments {}.into();
                    let witness = case.into_witness(witness.clone());

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U256TestDivArguments, U256TestDivWitness, _>::new()
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
    fn calculate_normalizer_base_128(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let threshold = U256::one() << 127;
        let strategy = {
            let a = arb_u256_in_range(U256::one() << 128, U256::MAX);

            a.prop_map(move |a| {
                let high = (a >> 128).as_u128();
                let normalizer = threshold.as_u128().div_ceil(high);

                FuzzCase::arg(a).expect(U256::from(normalizer))
            })
        };

        case_fuzz(
            CalculateNormalizerBase128,
            fuzz_engine_builder,
            "u256 calculate base 128 normalizer",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn calculate_normalizer_base_128_norm_is_1(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let threshold = U256::one() << 127;
        let strategy = {
            let a = arb_u256_in_range(U256::one() << 255, U256::MAX);

            a.prop_map(move |a| {
                let high = (a >> 128).as_u128();
                let normalizer = threshold.as_u128().div_ceil(high);

                FuzzCase::arg(a).expect(U256::from(normalizer))
            })
        };

        case_fuzz(
            CalculateNormalizerBase128,
            fuzz_engine_builder,
            "u256 calculate base 128 normalizer is one",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn calculate_normalizer_base_128_norm_greater_than_1(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let threshold = U256::one() << 127;
        let strategy = {
            let a = arb_u256_in_range(U256::one() << 128, (U256::one() << 255) - 1);

            a.prop_map(move |a| {
                let high = (a >> 128).as_u128();
                let normalizer = threshold.as_u128().div_ceil(high);

                FuzzCase::arg(a).expect(U256::from(normalizer))
            })
        };

        case_fuzz(
            CalculateNormalizerBase128,
            fuzz_engine_builder,
            "u256 calculate base 128 normalizer is greater than one",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn calculate_normalizer_base_128_a_is_u128_fail(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::one(), (U256::one() << 128) - 1);

            a.prop_map(FuzzCase::arg)
        };

        case_fuzz(
            CalculateNormalizerBase128,
            fuzz_engine_builder,
            "u256 calculate base 128 normalizer rejects u128",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn calculate_normalizer_base_128_b_is_zero_fail(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = Just(U256::zero()).prop_map(FuzzCase::arg);

        case_fuzz(
            CalculateNormalizerBase128,
            fuzz_engine_builder,
            "u256 calculate base 128 normalizer rejects zero",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_64(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_non_zero_64bit_u256();

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b).second(a % b))
        };

        case_fuzz(DivMod256_64, fuzz_engine_builder, "u256 div mod by u64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_64_overflow(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(FuzzCase::arg);

        case_fuzz(
            DivMod256_64,
            fuzz_engine_builder,
            "u256 div mod by u64 zero",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn algorithm_d_256_128(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_u256_in_range(U256::from(u64::MAX) + 1, U256::from(u128::MAX));

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b).second(a % b))
        };

        case_fuzz(
            AlgorithmD256_128,
            fuzz_engine_builder,
            "u256 algorithm d by u128",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn algorithm_d_256_128_fail_b_fits_into_u64(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_non_zero_64bit_u256();

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b).second(a % b))
        };

        case_fuzz(
            AlgorithmD256_128,
            fuzz_engine_builder,
            "u256 algorithm d rejects u64 divisor",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn algorithm_d_256_128_overflow(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(FuzzCase::arg);

        case_fuzz(
            AlgorithmD256_128,
            fuzz_engine_builder,
            "u256 algorithm d rejects zero divisor",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn algorithm_d_256_128_a_eq_b(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_128bit_u256();

            a.prop_map(|a| FuzzCase::new(a, a).expect(U256::one()))
        };

        case_fuzz(
            AlgorithmD256_128,
            fuzz_engine_builder,
            "u256 algorithm d equal arguments",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_128(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_u256_in_range(U256::from(u64::MAX) + 1, U256::from(u128::MAX));

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b).second(a % b))
        };

        case_fuzz(DivMod256_128, fuzz_engine_builder, "u256 div mod by u128")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_128_b_fits_into_u64(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_non_zero_64bit_u256();

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b).second(a % b))
        };

        case_fuzz(
            DivMod256_128,
            fuzz_engine_builder,
            "u256 div mod by u64 through u128",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_128_overflow(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(FuzzCase::arg);

        case_fuzz(
            DivMod256_128,
            fuzz_engine_builder,
            "u256 div mod by u128 zero",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_128_a_eq_b(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_128bit_u256();

            a.prop_map(|a| FuzzCase::new(a, a).expect(U256::one()))
        };

        case_fuzz(
            DivMod256_128,
            fuzz_engine_builder,
            "u256 div mod by u128 equal arguments",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_a_less_than_b(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::one(), U256::MAX - 1);

            a.prop_flat_map(|a| {
                arb_u256_in_range(a + 1, U256::MAX)
                    .prop_map(move |b| FuzzCase::new(a, b).expect(a / b).second(a % b))
            })
        };

        case_fuzz(DivMod256, fuzz_engine_builder, "u256 div mod a less than b")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_div_128(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let b = arb_non_zero_128bit_u256();

            b.prop_flat_map(|b| {
                arb_u256_in_range(b, U256::from(u128::MAX))
                    .prop_map(move |a| FuzzCase::new(a, b).expect(a / b).second(a % b))
            })
        };

        case_fuzz(DivMod256, fuzz_engine_builder, "u256 div mod u128 dividend")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_q_is_1(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let b_low = arb_128bit_u256();
            let high = arb_non_zero_128bit_u256();

            (b_low, high).prop_flat_map(|(b_low, high)| {
                arb_u256_in_range(b_low, U256::from(u128::MAX)).prop_map(move |a_low| {
                    let a = (high << 128) | a_low;
                    let b = (high << 128) | b_low;

                    FuzzCase::new(a, b).expect(a / b).second(a % b)
                })
            })
        };

        case_fuzz(
            DivMod256,
            fuzz_engine_builder,
            "u256 div mod quotient is one",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_b_fits_into_u128(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::from(u128::MAX) + 1, U256::MAX);
            let b = arb_non_zero_128bit_u256();

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b).second(a % b))
        };

        case_fuzz(DivMod256, fuzz_engine_builder, "u256 div mod u128 divisor")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_b_is_u256(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let b = arb_u256_in_range(U256::one(), U256::MAX - 1);

            b.prop_flat_map(|b| {
                arb_u256_in_range(b + 1, U256::MAX)
                    .prop_map(move |a| FuzzCase::new(a, b).expect(a / b).second(a % b))
            })
        };

        case_fuzz(DivMod256, fuzz_engine_builder, "u256 div mod u256 divisor")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_a_equal_b(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();

            a.prop_map(|a| FuzzCase::new(a, a).expect(U256::one()))
        };

        case_fuzz(
            DivMod256,
            fuzz_engine_builder,
            "u256 div mod equal arguments",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_equal_high_words_max_low_diff(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let high = arb_non_zero_128bit_u256();

            high.prop_map(|high| {
                let a = (high << 128) | U256::from(u128::MAX);
                let b = high << 128;

                FuzzCase::new(a, b)
                    .expect(U256::one())
                    .second(U256::from(u128::MAX))
            })
        };

        case_fuzz(
            DivMod256,
            fuzz_engine_builder,
            "u256 div mod equal high words maximum low difference",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_mod_256_eq_high_words_a_less_than_b(
        fuzz_engine_builder: U256DivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let high = arb_non_zero_128bit_u256();

            high.prop_map(|high| {
                let a = high << 128;
                let b = (high << 128) | U256::from(u128::MAX);

                FuzzCase::new(a, b).second(a)
            })
        };

        case_fuzz(
            DivMod256,
            fuzz_engine_builder,
            "u256 div mod equal high words a less than b",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn div_256(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();
            let b = arb_non_zero_u256();

            (a, b).prop_map(|(a, b)| FuzzCase::new(a, b).expect(a / b))
        };

        case_fuzz(Div256, fuzz_engine_builder, "u256 division")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn div_256_div_by_zero(fuzz_engine_builder: U256DivFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = arb_u256().prop_map(FuzzCase::arg);

        case_fuzz(Div256, fuzz_engine_builder, "u256 division by zero")
            .strategy(strategy)
            .run()
    }
}
