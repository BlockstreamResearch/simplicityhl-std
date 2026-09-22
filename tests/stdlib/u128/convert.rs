use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u128::convert::ConvertProgram as U128ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u128::convert::derived_convert::{
    ConvertArguments as U128ConvertTestArguments, ConvertWitness as U128ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    U128ToU256,
    SplitU128IntoU8,
    SplitU128IntoU16,
    SplitU128IntoU32,
    SplitU128IntoU64,
    SafeU128ToU1,
    SafeU128ToU8,
    SafeU128ToU16,
    SafeU128ToU32,
    SafeU128ToU64,
}

fn program() -> U128ConvertTestProgram {
    U128ConvertTestProgram::new(U128ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U128ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U128ConvertTestWitness {
            function_index: function as u8,
            first_arg: 0,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: u128) -> Self {
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

#[simplex::test]
fn u128_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(U128ToU256)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u128_into_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(SplitU128IntoU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u128_into_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(SplitU128IntoU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u128_into_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(SplitU128IntoU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u128_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u128::MAX);

    case(SplitU128IntoU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u128_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(SafeU128ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u128_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u128::MAX);

    case(SafeU128ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u128_to_u8(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX as u128);

    case(SafeU128ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u128_to_u8_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u8::MAX as u128 + 1..=u128::MAX);

    case(SafeU128ToU8)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u128_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u16::MAX as u128);

    case(SafeU128ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u128_to_u16_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u16::MAX as u128 + 1..=u128::MAX);

    case(SafeU128ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u128_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u32::MAX as u128);

    case(SafeU128ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u128_to_u32_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u32::MAX as u128 + 1..=u128::MAX);

    case(SafeU128ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn safe_u128_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u64::MAX as u128);

    case(SafeU128ToU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u128_to_u64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(u64::MAX as u128 + 1..=u128::MAX);

    case(SafeU128ToU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
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

    type Builder =
        FuzzEngineBuilder<U128ConvertTestProgram, U128ConvertTestArguments, U128ConvertTestWitness>;

    struct FuzzCaseBuilder {
        case: Case,
        builder: Builder,
        inputs: Option<BoxedStrategy<u128>>,
        expect: Expect,
        name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: Builder,
        name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            builder,
            inputs: None,
            expect: Expect::Ok,
            name,
        }
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = u128> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
            self
        }

        fn expect(mut self, expect: Expect) -> Self {
            self.expect = expect;
            self
        }

        fn run(self) -> anyhow::Result<()> {
            let witness = self.case.witness;
            let strategy = self
                .inputs
                .expect("a fuzz strategy must be specified")
                .prop_map(move |a| {
                    let arguments: Arguments = U128ConvertTestArguments {}.into();
                    let witness: WitnessValues = Case {
                        witness: witness.clone(),
                    }
                    .arg(a)
                    .expect(U256::from(a).to_big_endian())
                    .witness
                    .into();
                    (arguments, witness)
                });
            let strategy =
                FuzzStrategyBuilder::<U128ConvertTestArguments, U128ConvertTestWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();
            let mut tx = FinalTransaction::new();
            tx.add_input(PartialInput::new(UTXO::default()), RequiredSignature::None);
            let tx_builder = FinalTransactionBuilder::new(tx, [ProgramTarget::Input(0)])?;
            self.builder
                .build(strategy, tx_builder)
                .run_with_check(FuzzExecutionCheck::new(self.name, self.expect));
            Ok(())
        }
    }

    fn arb_bool() -> impl Strategy<Value = bool> {
        any::<bool>()
    }

    fn arb_u8() -> impl Strategy<Value = u8> {
        any::<u8>()
    }

    fn arb_u16() -> impl Strategy<Value = u16> {
        any::<u16>()
    }

    fn arb_u32() -> impl Strategy<Value = u32> {
        any::<u32>()
    }

    fn arb_u64() -> impl Strategy<Value = u64> {
        any::<u64>()
    }

    fn arb_u128() -> impl Strategy<Value = u128> {
        any::<u128>()
    }

    fn arb_bool_u128() -> impl Strategy<Value = u128> {
        arb_bool().prop_map(u128::from)
    }

    fn arb_u8_u128() -> impl Strategy<Value = u128> {
        arb_u8().prop_map(u128::from)
    }

    fn arb_u16_u128() -> impl Strategy<Value = u128> {
        arb_u16().prop_map(u128::from)
    }

    fn arb_u32_u128() -> impl Strategy<Value = u128> {
        arb_u32().prop_map(u128::from)
    }

    fn arb_u64_u128() -> impl Strategy<Value = u128> {
        arb_u64().prop_map(u128::from)
    }

    #[simplex::fuzz]
    fn u128_to_u256(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(U128ToU256, builder, "u128 to u256")
            .strategy(arb_u128())
            .run()
    }

    #[simplex::fuzz]
    fn split_u128_into_u8(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SplitU128IntoU8, builder, "split u128 into u8")
            .strategy(arb_u128())
            .run()
    }

    #[simplex::fuzz]
    fn split_u128_into_u16(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SplitU128IntoU16, builder, "split u128 into u16")
            .strategy(arb_u128())
            .run()
    }

    #[simplex::fuzz]
    fn split_u128_into_u32(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SplitU128IntoU32, builder, "split u128 into u32")
            .strategy(arb_u128())
            .run()
    }

    #[simplex::fuzz]
    fn split_u128_into_u64(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SplitU128IntoU64, builder, "split u128 into u64")
            .strategy(arb_u128())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u1(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU1, builder, "safe u128 to u1")
            .strategy(arb_bool_u128())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u1_overflow(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU1, builder, "safe u128 to u1 overflow")
            .strategy(2u128..=u128::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u8(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU8, builder, "safe u128 to u8")
            .strategy(arb_u8_u128())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u8_overflow(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU8, builder, "safe u128 to u8 overflow")
            .strategy((u128::from(u8::MAX) + 1)..=u128::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u16(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU16, builder, "safe u128 to u16")
            .strategy(arb_u16_u128())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u16_overflow(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU16, builder, "safe u128 to u16 overflow")
            .strategy((u128::from(u16::MAX) + 1)..=u128::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u32(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU32, builder, "safe u128 to u32")
            .strategy(arb_u32_u128())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u32_overflow(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU32, builder, "safe u128 to u32 overflow")
            .strategy((u128::from(u32::MAX) + 1)..=u128::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u64(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU64, builder, "safe u128 to u64")
            .strategy(arb_u64_u128())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u128_to_u64_overflow(builder: Builder) -> anyhow::Result<()> {
        case_fuzz(SafeU128ToU64, builder, "safe u128 to u64 overflow")
            .strategy((u128::from(u64::MAX) + 1)..=u128::MAX)
            .expect(Expect::AssertFailed)
            .run()
    }
}
