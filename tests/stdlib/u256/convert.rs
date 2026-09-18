use primitive_types::U256;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u256::convert::ConvertProgram as U256ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u256::convert::derived_convert::{
    ConvertArguments as U256ConvertTestArguments, ConvertWitness as U256ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    SplitU256IntoU8,
    SplitU256IntoU16,
    SplitU256IntoU32,
    SplitU256IntoU64,
    SplitU256IntoU128,
    SafeU256ToU1,
    SafeU256ToU8,
    SafeU256ToU16,
    SafeU256ToU32,
    SafeU256ToU64,
    SafeU256ToU128,
}

fn program() -> U256ConvertTestProgram {
    U256ConvertTestProgram::new(U256ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256ConvertTestWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: [u8; 32]) -> Self {
        self.witness.first_arg = a;
        self
    }

    /// `expected`: the value the arm should produce.
    fn expect(mut self, expected: [u8; 32]) -> Self {
        self.witness.expected = expected;
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

use crate::common::helper::generate_u256;

#[simplex::test]
fn u256_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU8)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU16)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU32)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU64)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u256_into_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX);

    case(SplitU256IntoU128)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::one());

    case(SafeU256ToU1)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(2), U256::MAX);

    case(SafeU256ToU1)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u8::MAX));

    case(SafeU256ToU8)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u8::MAX) + 1, U256::MAX);

    case(SafeU256ToU8)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u16::MAX));

    case(SafeU256ToU16)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u16::MAX) + 1, U256::MAX);

    case(SafeU256ToU16)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u32::MAX));

    case(SafeU256ToU32)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u32_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u32::MAX) + 1, U256::MAX);

    case(SafeU256ToU32)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u64::MAX));

    case(SafeU256ToU64)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u64::MAX) + 1, U256::MAX);

    case(SafeU256ToU64)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u256_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::from(u128::MAX));

    case(SafeU256ToU128)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u256_to_u128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);

    case(SafeU256ToU128)
        .arg(a.to_big_endian())
        .expect(a.to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

mod convert_tests_fuzz {
    use super::*;

    use crate::common::core::FuzzExecutionCheck;
    use simplex::fuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::builders::{FinalTransactionBuilder, ProgramTarget};
    use simplex::fuzz::engine::FuzzStrategyBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::simplicityhl::{Arguments, WitnessValues};
    use simplex::transaction::{FinalTransaction, PartialInput, RequiredSignature, UTXO};

    const PROGRAM_TARGET: ProgramTarget = ProgramTarget::Input(0);
    type U256ConvertFuzzEngineBuilder =
        FuzzEngineBuilder<U256ConvertTestProgram, U256ConvertTestArguments, U256ConvertTestWitness>;

    // (A, Result)
    type ConvertInputs = (U256, U256);

    fn arb_u256() -> impl Strategy<Value = U256> {
        any::<[u8; 32]>().prop_map(|bytes| U256::from_big_endian(&bytes))
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
    fn split_number<T: Clone>(n: T) -> (T, T) {
        (n.clone(), n)
    }

    fn u256_duplicated_strategy() -> impl Strategy<Value = ConvertInputs> {
        arb_u256().prop_map(|a| split_number(a))
    }

    fn safe_u256_to_u1_strategy() -> impl Strategy<Value = U256> {
        any::<bool>().prop_map(|x| U256::from(x as u8))
    }

    fn safe_u256_to_u8_strategy() -> impl Strategy<Value = U256> {
        any::<u8>().prop_map(U256::from)
    }

    fn safe_u256_to_u16_strategy() -> impl Strategy<Value = U256> {
        any::<u16>().prop_map(U256::from)
    }

    fn safe_u256_to_u32_strategy() -> impl Strategy<Value = U256> {
        any::<u32>().prop_map(U256::from)
    }

    fn safe_u256_to_u64_strategy() -> impl Strategy<Value = U256> {
        any::<u64>().prop_map(U256::from)
    }

    fn safe_u256_to_u128_strategy() -> impl Strategy<Value = U256> {
        any::<u128>().prop_map(U256::from)
    }

    fn overflow_strategy(max: U256) -> impl Strategy<Value = U256> {
        arb_u256_in_range(max + U256::one(), U256::MAX)
    }

    struct CaseFuzz {
        case: Case,
        builder: U256ConvertFuzzEngineBuilder,
        inputs: Option<BoxedStrategy<ConvertInputs>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: U256ConvertFuzzEngineBuilder,
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
        fn strategy(mut self, inputs: impl Strategy<Value = ConvertInputs> + 'static) -> Self {
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
                .prop_map(move |(a, expected)| (a.to_big_endian(), expected.to_big_endian()))
                .prop_map(move |(a, expected)| {
                    let arguments: Arguments = U256ConvertTestArguments {}.into();
                    let witness: WitnessValues = Case {
                        witness: witness.clone(),
                    }
                    .arg(a)
                    .expect(expected)
                    .witness
                    .into();

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<U256ConvertTestArguments, U256ConvertTestWitness, _>::new()
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
    fn u256_into_u8(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = u256_duplicated_strategy();

        case_fuzz(SplitU256IntoU8, fuzz_engine_builder, "u256 split into u8")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn u256_into_u16(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = u256_duplicated_strategy();

        case_fuzz(SplitU256IntoU16, fuzz_engine_builder, "u256 split into u16")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn u256_into_u32(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = u256_duplicated_strategy();

        case_fuzz(SplitU256IntoU32, fuzz_engine_builder, "u256 split into u32")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn u256_into_u64(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = u256_duplicated_strategy();

        case_fuzz(SplitU256IntoU64, fuzz_engine_builder, "u256 split into u64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn u256_into_u128(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = u256_duplicated_strategy();

        case_fuzz(
            SplitU256IntoU128,
            fuzz_engine_builder,
            "u256 split into u128",
        )
        .strategy(strategy)
        .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u1(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = safe_u256_to_u1_strategy().prop_map(split_number);

        case_fuzz(SafeU256ToU1, fuzz_engine_builder, "safe u256 to u1")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u1_overflow(
        fuzz_engine_builder: U256ConvertFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = overflow_strategy(U256::one()).prop_map(split_number);

        case_fuzz(
            SafeU256ToU1,
            fuzz_engine_builder,
            "safe u256 to u1 overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u8(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = safe_u256_to_u8_strategy().prop_map(split_number);

        case_fuzz(SafeU256ToU8, fuzz_engine_builder, "safe u256 to u8")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u8_overflow(
        fuzz_engine_builder: U256ConvertFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = overflow_strategy(u8::MAX.into()).prop_map(split_number);

        case_fuzz(
            SafeU256ToU8,
            fuzz_engine_builder,
            "safe u256 to u8 overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u16(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = safe_u256_to_u16_strategy().prop_map(split_number);

        case_fuzz(SafeU256ToU16, fuzz_engine_builder, "safe u256 to u16")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u16_overflow(
        fuzz_engine_builder: U256ConvertFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = overflow_strategy(u16::MAX.into()).prop_map(split_number);

        case_fuzz(
            SafeU256ToU16,
            fuzz_engine_builder,
            "safe u256 to u16 overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u32(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = safe_u256_to_u32_strategy().prop_map(split_number);

        case_fuzz(SafeU256ToU32, fuzz_engine_builder, "safe u256 to u32")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u32_overflow(
        fuzz_engine_builder: U256ConvertFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = overflow_strategy(u32::MAX.into()).prop_map(split_number);

        case_fuzz(
            SafeU256ToU32,
            fuzz_engine_builder,
            "safe u256 to u32 overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u64(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = safe_u256_to_u64_strategy().prop_map(split_number);

        case_fuzz(SafeU256ToU64, fuzz_engine_builder, "safe u256 to u64")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u64_overflow(
        fuzz_engine_builder: U256ConvertFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = overflow_strategy(u64::MAX.into()).prop_map(split_number);

        case_fuzz(
            SafeU256ToU64,
            fuzz_engine_builder,
            "safe u256 to u64 overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u128(fuzz_engine_builder: U256ConvertFuzzEngineBuilder) -> anyhow::Result<()> {
        let strategy = safe_u256_to_u128_strategy().prop_map(split_number);

        case_fuzz(SafeU256ToU128, fuzz_engine_builder, "safe u256 to u128")
            .strategy(strategy)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u256_to_u128_overflow(
        fuzz_engine_builder: U256ConvertFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        let strategy = overflow_strategy(u128::MAX.into()).prop_map(split_number);

        case_fuzz(
            SafeU256ToU128,
            fuzz_engine_builder,
            "safe u256 to u128 overflow",
        )
        .strategy(strategy)
        .expect(Expect::AssertFailed)
        .run()
    }
}
