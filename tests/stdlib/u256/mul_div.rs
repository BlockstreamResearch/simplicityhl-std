use primitive_types::{U256, U512};
use std::cmp::max;
use std::ops::Div;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::mul_div::MulDivProgram as U256MulDivTestProgram;
use simplicityhl_std::artifacts::tests::u256::mul_div::derived_mul_div::{
    MulDivArguments as U256MulDivTestArguments, MulDivWitness as U256MulDivTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    MulDiv,
}

fn program() -> U256MulDivTestProgram {
    U256MulDivTestProgram::new(U256MulDivTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256MulDivTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256MulDivTestWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            third_arg: [0; 32],
            expected: None,
        },
    }
}

impl Case {
    /// The three operands, `first_arg`, `second_arg` and `third_arg`.
    fn args(mut self, a: [u8; 32], b: [u8; 32], c: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self.witness.second_arg = b;
        self.witness.third_arg = c;
        self
    }

    /// The value the arm should return. `None`, the default, means the arm is
    /// expected to produce nothing.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = Some(expected);
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

fn safe_u512_to_u256(a: [u8; 64]) -> [u8; 32] {
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    assert_eq!(high, U256::zero());

    low.to_big_endian()
}

fn safe_u512_to_u256_typed(a: U512) -> U256 {
    let a = a.to_big_endian();
    let high = U256::from_big_endian(&a[0..32]);
    let low = U256::from_big_endian(&a[32..64]);

    assert_eq!(high, U256::zero());

    low
}

#[simplex::test]
fn mul_div_256_product_fits_into_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u128::MAX));
    let b = generate_u256(U256::zero(), U256::from(u128::MAX));
    let c = generate_u256(U256::one(), U256::MAX);

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_intermediate_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(2), U256::MAX);
    let b = U256::MAX;
    let c = generate_u256(a, U256::MAX);

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_result_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(2), U256::MAX);
    let b = U256::MAX;
    let c = generate_u256(U256::one(), a);

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect([0; 32])
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn mul_div_256_remainder_is_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let b = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let c = a;

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(b.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn mul_div_256_denominator_is_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let c = generate_u256(b, U256::from(u128::MAX));

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_min_denom_high(context: simplex::TestContext) -> anyhow::Result<()> {
    let pow129: U256 = U256::from(2).pow(U256::from(129));

    let a = generate_u256(U256::from(u128::MAX) + 1, pow129);
    let b = generate_u256(U256::from(u128::MAX) + 1, pow129);
    let c = generate_u256(U256::from(u128::MAX) + 1, pow129 - 1);

    let res = safe_u512_to_u256(a.full_mul(b).div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_div_by_zero(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::one(), U256::MAX);
    let b = generate_u256(U256::one(), U256::MAX);
    let c = U256::zero();

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect([0; 32])
        .run(&context)
}

#[simplex::test]
fn mul_div_256_algorithm_d_512_256_check(context: simplex::TestContext) -> anyhow::Result<()> {
    let pow2_128: U256 = U256::from(u128::MAX) + 1;

    let a = generate_u256(pow2_128, U256::MAX);
    let b = generate_u256(pow2_128, U256::MAX - pow2_128);

    let product = a.full_mul(b);
    let result_high = U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

    let c = generate_u256(max(U256::from(u128::MAX) + 1, result_high + 1), U256::MAX);

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_algorithm_d_512_256_c_is_res_high(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let pow2_128: U256 = U256::from(u128::MAX) + 1;

    let a: U256 = generate_u256(pow2_128, U256::MAX);
    let b = generate_u256(pow2_128, U256::MAX - pow2_128);

    let product = a.full_mul(b);
    let result_high = U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

    let c = result_high + 1;

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_normalize_to_threshold_512_127_norm_is_1(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let pow2_128: U256 = U256::from(u128::MAX) + 1;

    let a: U256 = generate_u256(pow2_128, U256::MAX);
    let b = generate_u256(pow2_128, U256::MAX - pow2_128);

    let product = a.full_mul(b);
    let result_high = U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian())));

    let c = generate_u256(
        max(U256::from(2).pow(U256::from(255)), result_high + 1),
        U256::MAX,
    );

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

#[simplex::test]
fn mul_div_256_normalize_to_threshold_512_127_norm_greater_than_1(
    context: simplex::TestContext,
) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);
    let b = generate_u256(
        U256::from(u128::MAX) + 1,
        U256::from(2).pow(U256::from(192)),
    );

    let product = a.full_mul(b);
    let result_high =
        U256::from_big_endian(&(safe_u512_to_u256((product >> 256).to_big_endian()))) + 1;

    let c = generate_u256(
        max(U256::from(u128::MAX) + 1, result_high),
        U256::from(2).pow(U256::from(255)) - 1,
    );

    let res = safe_u512_to_u256(product.div(c).to_big_endian());

    case(MulDiv)
        .args(a.to_big_endian(), b.to_big_endian(), c.to_big_endian())
        .expect(res)
        .run(&context)
}

mod mul_div_tests_fuzz {
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

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type U256MulDivFuzzEngineBuilder =
        FuzzEngineBuilder<U256MulDivTestProgram, U256MulDivTestArguments, U256MulDivTestWitness>;

    // (A, B, C, Result)
    type MulDivInputs = (U256, U256, U256, U256);

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

    #[inline]
    fn product_high(a: U256, b: U256) -> U256 {
        safe_u512_to_u256_typed(a.full_mul(b) >> 256)
    }

    struct CaseFuzz {
        case: Case,
        builder: U256MulDivFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<MulDivInputs>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U256MulDivFuzzEngineBuilder,
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
        fn strategy(mut self, inputs: impl Strategy<Value = MulDivInputs> + 'static) -> Self {
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
            let expect_failure = matches!(self.expect, Expect::AssertFailed);

            let strategy = inputs
                .prop_map(move |(a, b, c, expected)| {
                    (
                        a.to_big_endian(),
                        b.to_big_endian(),
                        c.to_big_endian(),
                        expected.to_big_endian(),
                    )
                })
                .prop_map(move |(a, b, c, expected)| {
                    let arguments: Arguments = U256MulDivTestArguments {}.into();
                    let case = Case {
                        witness: witness.clone(),
                    }
                    .args(a, b, c);
                    let case = if expect_failure {
                        case
                    } else {
                        case.expect(expected)
                    };
                    let witness: WitnessValues = case.witness.into();

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U256MulDivTestArguments, U256MulDivTestWitness, _>::new()
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
    fn mul_div_256_product_fits_into_u256(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_128bit_u256();
            let b = arb_128bit_u256();
            let c = arb_non_zero_u256();

            (a, b, c).prop_map(|(a, b, c)| {
                let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));
                (a, b, c, expected)
            })
        };

        case_fuzz(
            MulDiv,
            fuzz_engine_builder,
            "mul_div_256 product fits into u256",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_intermediate_overflow(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::from(2), U256::MAX);
            let b = Just(U256::MAX);

            (a, b).prop_flat_map(|(a, b)| {
                arb_u256_in_range(a, U256::MAX).prop_map(move |c| {
                    let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));
                    (a, b, c, expected)
                })
            })
        };

        case_fuzz(
            MulDiv,
            fuzz_engine_builder,
            "mul_div_256 intermediate overflow",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_result_overflow(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::from(2), U256::MAX);
            let b = Just(U256::MAX);

            (a, b).prop_flat_map(|(a, b)| {
                arb_u256_in_range(U256::one(), a - U256::one())
                    .prop_map(move |c| (a, b, c, U256::zero()))
            })
        };

        case_fuzz(MulDiv, fuzz_engine_builder, "mul_div_256 result overflow")
            .strategy(strategy)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_remainder_is_zero(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let pow2_128 = U256::one() << 128;
        let strategy = {
            let a = arb_u256_in_range(pow2_128, U256::MAX);
            let b = arb_u256_in_range(pow2_128, U256::MAX);

            (a, b).prop_map(|(a, b)| (a, b, a, b))
        };

        case_fuzz(MulDiv, fuzz_engine_builder, "mul_div_256 remainder is zero")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_denominator_is_u128(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();
            let b = arb_non_zero_128bit_u256();

            (a, b).prop_flat_map(|(a, b)| {
                arb_u256_in_range(b, U256::from(u128::MAX)).prop_map(move |c| {
                    let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));

                    (a, b, c, expected)
                })
            })
        };

        case_fuzz(
            MulDiv,
            fuzz_engine_builder,
            "mul_div_256 denominator is u128",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_min_denom_high(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let pow2_128 = U256::one() << 128;
        let pow2_129 = U256::one() << 129;
        let strategy = {
            let a = arb_u256_in_range(pow2_128, pow2_129);
            let b = arb_u256_in_range(pow2_128, pow2_129);
            let c = arb_u256_in_range(pow2_128, pow2_129 - U256::one());

            (a, b, c).prop_map(|(a, b, c)| {
                let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));

                (a, b, c, expected)
            })
        };

        case_fuzz(
            MulDiv,
            fuzz_engine_builder,
            "mul_div_256 minimal denominator high",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_div_by_zero(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();
            let b = arb_non_zero_u256();

            (a, b).prop_map(|(a, b)| (a, b, U256::zero(), U256::zero()))
        };

        case_fuzz(MulDiv, fuzz_engine_builder, "mul_div_256 division by zero")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_algorithm_d_512_256_check(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let pow2_128 = U256::one() << 128;
        let strategy = {
            let a = arb_u256_in_range(pow2_128, U256::MAX);
            let b = arb_u256_in_range(pow2_128, U256::MAX - pow2_128);

            (a, b).prop_flat_map(move |(a, b)| {
                let min_c = max(pow2_128, product_high(a, b) + U256::one());

                arb_u256_in_range(min_c, U256::MAX).prop_map(move |c| {
                    let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));

                    (a, b, c, expected)
                })
            })
        };

        case_fuzz(MulDiv, fuzz_engine_builder, "mul_div_256 algorithm D")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_algorithm_d_512_256_c_is_res_high(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let pow2_128 = U256::one() << 128;

        let strategy = {
            let a = arb_u256_in_range(pow2_128, U256::MAX);
            let b = arb_u256_in_range(pow2_128, U256::MAX - pow2_128);

            (a, b).prop_map(|(a, b)| {
                let c = product_high(a, b) + U256::one();

                let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));

                (a, b, c, expected)
            })
        };

        case_fuzz(
            MulDiv,
            fuzz_engine_builder,
            "mul_div_256 denominator just above product high",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_normalize_to_threshold_512_127_norm_is_1(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let pow2_128 = U256::one() << 128;
        let pow2_255 = U256::one() << 255;

        let strategy = {
            let a = arb_u256_in_range(pow2_128, U256::MAX);
            let b = arb_u256_in_range(pow2_128, U256::MAX - pow2_128);

            (a, b).prop_flat_map(move |(a, b)| {
                let min_c = max(pow2_255, product_high(a, b) + U256::one());

                arb_u256_in_range(min_c, U256::MAX).prop_map(move |c| {
                    let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));
                    (a, b, c, expected)
                })
            })
        };

        case_fuzz(MulDiv, fuzz_engine_builder, "mul_div_256 normalizer is one")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn mul_div_256_normalize_to_threshold_512_127_norm_greater_than_1(
        fuzz_engine_builder: U256MulDivFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let pow2_128 = U256::one() << 128;
        let pow2_192 = U256::one() << 192;
        let pow2_255 = U256::one() << 255;

        let strategy = {
            let a = arb_u256_in_range(pow2_128, U256::MAX);
            let b = arb_u256_in_range(pow2_128, pow2_192);

            (a, b).prop_flat_map(move |(a, b)| {
                let min_c = max(pow2_128, product_high(a, b) + U256::one());

                arb_u256_in_range(min_c, pow2_255 - U256::one()).prop_map(move |c| {
                    let expected = safe_u512_to_u256_typed(a.full_mul(b).div(c));
                    (a, b, c, expected)
                })
            })
        };

        case_fuzz(
            MulDiv,
            fuzz_engine_builder,
            "mul_div_256 normalizer is greater than one",
        )
        .strategy(strategy)
        .run()
    }
}
