use primitive_types::U256;
use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::u8::convert::ConvertProgram as U8ConvertTestProgram;
use simplicityhl_std::artifacts::tests::u8::convert::derived_convert::{
    ConvertArguments as U8ConvertTestArguments, ConvertWitness as U8ConvertTestWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    U8ToU16,
    U8ToU32,
    U8ToU64,
    U8ToU128,
    U8ToU256,
    SplitU8IntoU1,
    SafeU8ToU1,
}

fn program() -> U8ConvertTestProgram {
    U8ConvertTestProgram::new(&U8ConvertTestArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U8ConvertTestWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U8ConvertTestWitness {
            function_index: function as u8,
            first_arg: 0,
            expected: [0; 32],
        },
    }
}

impl Case {
    /// The operand, `first_arg`.
    fn arg(mut self, a: u8) -> Self {
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
fn u8_to_u16(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX);

    case(U8ToU16)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u8_to_u32(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX);

    case(U8ToU32)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u8_to_u64(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX);

    case(U8ToU64)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u8_to_u128(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX);

    case(U8ToU128)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn u8_to_u256(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX);

    case(U8ToU256)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn split_u8_into_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=u8::MAX);

    case(SplitU8IntoU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u8_to_u1(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(0..=1);

    case(SafeU8ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .run(&context)
}

#[simplex::test]
fn safe_u8_to_u1_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = rand::thread_rng().gen_range(2..=u8::MAX);

    case(SafeU8ToU1)
        .arg(a)
        .expect(U256::from(a).to_big_endian())
        .expecting(&context, Expect::AssertFailed)
}

mod convert_tests_fuzz {
    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::Expect;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

    struct FuzzCaseBuilder {
        case: Case,
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<u8>>,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        context: TestContext<FuzzMode>,
        test_name: &'static str,
    ) -> FuzzCaseBuilder {
        FuzzCaseBuilder {
            case: case(function),
            context,
            inputs: None,
            test_name,
            expect: Expect::Accept,
        }
    }

    fn arb_u8() -> impl Strategy<Value = u8> {
        any::<u8>()
    }

    fn arb_bool() -> impl Strategy<Value = bool> {
        any::<bool>()
    }

    fn arb_bool_u8() -> impl Strategy<Value = u8> {
        arb_bool().prop_map(u8::from)
    }

    impl FuzzCaseBuilder {
        fn strategy(mut self, inputs: impl Strategy<Value = u8> + 'static) -> Self {
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
                .prop_map(move |a| {
                    let arguments: Arguments = U8ConvertTestArguments {}.into();

                    let witness: WitnessValues = Case {
                        witness: witness.clone(),
                    }
                    .arg(a)
                    .expect(U256::from(a).to_big_endian())
                    .witness
                    .into();

                    (arguments, witness)
                })
                .boxed();

            crate::common::fuzz::run::<U8ConvertTestProgram>(
                &self.context,
                strategy,
                self.test_name,
                self.expect,
            );

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn u8_to_u16(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U8ToU16, context, "u8 to u16")
            .strategy(arb_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u8_to_u32(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U8ToU32, context, "u8 to u32")
            .strategy(arb_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u8_to_u64(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U8ToU64, context, "u8 to u64")
            .strategy(arb_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u8_to_u128(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U8ToU128, context, "u8 to u128")
            .strategy(arb_u8())
            .run()
    }

    #[simplex::fuzz]
    fn u8_to_u256(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(U8ToU256, context, "u8 to u256")
            .strategy(arb_u8())
            .run()
    }

    #[simplex::fuzz]
    fn split_u8_into_u1(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SplitU8IntoU1, context, "split u8 into u1")
            .strategy(arb_u8())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u8_to_u1(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU8ToU1, context, "safe u8 to u1")
            .strategy(arb_bool_u8())
            .run()
    }

    #[simplex::fuzz]
    fn safe_u8_to_u1_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        case_fuzz(SafeU8ToU1, context, "safe u8 to u1 overflow")
            .strategy(2u8..=u8::MAX)
            .expect(Expect::Reject)
            .run()
    }
}
