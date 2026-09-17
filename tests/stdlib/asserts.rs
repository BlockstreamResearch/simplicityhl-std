use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::asserts::AssertsProgram as AssertsTestProgram;
use simplicityhl_std::artifacts::tests::asserts::derived_asserts::{
    AssertsArguments as AssertsTestArguments, AssertsWitness as AssertsTestWitness,
};

// Dispatch indices which must match the `is_selected(N, ..)` arms in
// simf/tests/asserts.simf.
use FunctionToTest::*;

#[derive(Clone, Copy)]
enum FunctionToTest {
    AssertEq1,
    AssertEq8,
    AssertEq16,
    AssertEq32,
    AssertEq64,
    AssertEq128,
    AssertEq256,
    AssertEqBool,

    AssertNone1,
    AssertNone8,
    AssertNone16,
    AssertNone32,
    AssertNone64,
    AssertNone128,
    AssertNone256,
}

const DEFAULT_SOME_U8: Option<u8> = Some(0);
const DEFAULT_SOME_U16: Option<u16> = Some(0);
const DEFAULT_SOME_U32: Option<u32> = Some(0);
const DEFAULT_SOME_U64: Option<u64> = Some(0);
const DEFAULT_SOME_U128: Option<u128> = Some(0);
const DEFAULT_SOME_U256: Option<[u8; 32]> = Some([0; 32]);

fn program() -> AssertsTestProgram {
    AssertsTestProgram::new(AssertsTestArguments {})
}

/// Returns two values in `[min, max]` that are equal when `same`, distinct otherwise.
pub fn generate_uints_in_one_range(same: bool, min_val: u128, max_val: u128) -> (u128, u128) {
    let some_u = rand::thread_rng().gen_range(min_val..=max_val);

    if same {
        return (some_u, some_u);
    }

    assert!(
        min_val != max_val,
        "cannot generate distinct values in a single-value range"
    );

    let mut other_u = rand::thread_rng().gen_range(min_val..=max_val);

    while other_u == some_u {
        other_u = rand::thread_rng().gen_range(min_val..=max_val);
    }

    (some_u, other_u)
}

fn default_witness(function_index: u8) -> AssertsTestWitness {
    AssertsTestWitness {
        function_index,
        first_arg_u1: DEFAULT_SOME_U8, // u1 in Simplicity is represented as u8
        second_arg_u1: DEFAULT_SOME_U8,
        first_arg_u8: DEFAULT_SOME_U8,
        second_arg_u8: DEFAULT_SOME_U8,
        first_arg_u16: DEFAULT_SOME_U16,
        second_arg_u16: DEFAULT_SOME_U16,
        first_arg_u32: DEFAULT_SOME_U32,
        second_arg_u32: DEFAULT_SOME_U32,
        first_arg_u64: DEFAULT_SOME_U64,
        second_arg_u64: DEFAULT_SOME_U64,
        first_arg_u128: DEFAULT_SOME_U128,
        second_arg_u128: DEFAULT_SOME_U128,
        first_arg_u256: DEFAULT_SOME_U256,
        second_arg_u256: DEFAULT_SOME_U256,
    }
}

/// Builds the witness for one assert call. `same` controls the two `assert_eq`
/// args; `none` makes the single `assert_none` arg `None`.
fn build_witness(function: FunctionToTest, same: bool, none: bool) -> AssertsTestWitness {
    let mut witness = default_witness(function as u8);

    match function {
        FunctionToTest::AssertEq1 => {
            let (a, b) = generate_uints_in_one_range(same, 0, 1u128);
            (witness.first_arg_u1, witness.second_arg_u1) = (Some(a as u8), Some(b as u8));
        }
        FunctionToTest::AssertEq8 => {
            let (a, b) = generate_uints_in_one_range(same, 0, u8::MAX as u128);
            (witness.first_arg_u8, witness.second_arg_u8) = (Some(a as u8), Some(b as u8));
        }
        FunctionToTest::AssertEq16 => {
            let (a, b) = generate_uints_in_one_range(same, 0, u16::MAX as u128);
            (witness.first_arg_u16, witness.second_arg_u16) = (Some(a as u16), Some(b as u16));
        }
        FunctionToTest::AssertEq32 => {
            let (a, b) = generate_uints_in_one_range(same, 0, u32::MAX as u128);
            (witness.first_arg_u32, witness.second_arg_u32) = (Some(a as u32), Some(b as u32));
        }
        FunctionToTest::AssertEq64 => {
            let (a, b) = generate_uints_in_one_range(same, 0, u64::MAX as u128);
            (witness.first_arg_u64, witness.second_arg_u64) = (Some(a as u64), Some(b as u64));
        }
        FunctionToTest::AssertEq128 => {
            let (a, b) = generate_uints_in_one_range(same, 0, u128::MAX);
            (witness.first_arg_u128, witness.second_arg_u128) = (Some(a), Some(b));
        }
        FunctionToTest::AssertEq256 => {
            let (a, b) = generate_uints_in_one_range(same, 0, u8::MAX as u128);
            (witness.first_arg_u256, witness.second_arg_u256) =
                (Some([a as u8; 32]), Some([b as u8; 32]));
        }
        FunctionToTest::AssertEqBool => {
            let (a, b) = generate_uints_in_one_range(same, 0, 1u128);
            (witness.first_arg_u1, witness.second_arg_u1) = (Some(a as u8), Some(b as u8));
        }
        FunctionToTest::AssertNone1 => {
            if none {
                witness.first_arg_u1 = None;
            }
        }
        FunctionToTest::AssertNone8 => {
            if none {
                witness.first_arg_u8 = None;
            }
        }
        FunctionToTest::AssertNone16 => {
            if none {
                witness.first_arg_u16 = None;
            }
        }
        FunctionToTest::AssertNone32 => {
            if none {
                witness.first_arg_u32 = None;
            }
        }
        FunctionToTest::AssertNone64 => {
            if none {
                witness.first_arg_u64 = None;
            }
        }
        FunctionToTest::AssertNone128 => {
            if none {
                witness.first_arg_u128 = None;
            }
        }
        FunctionToTest::AssertNone256 => {
            if none {
                witness.first_arg_u256 = None;
            }
        }
    }

    witness
}

/// One assert call, described by what makes it pass.
struct Case {
    function: FunctionToTest,
    same: bool,
    none: bool,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        function,
        same: false,
        none: false,
    }
}

impl Case {
    /// Make the two `assert_eq` arguments equal.
    fn equal(mut self) -> Self {
        self.same = true;
        self
    }

    /// Make the `assert_none` argument `None`.
    fn none(mut self) -> Self {
        self.none = true;
        self
    }

    /// Fund, spend, and expect the spend to succeed.
    fn run(self, context: &simplex::TestContext) -> anyhow::Result<()> {
        self.expecting(context, Expect::Ok)
    }

    /// Fund, spend, and expect `expect`.
    fn expecting(self, context: &simplex::TestContext, expect: Expect) -> anyhow::Result<()> {
        let witness = build_witness(self.function, self.same, self.none);
        run(context, program(), witness, expect)
    }
}

// assert_eq: happy = equal args, unhappy = distinct args
#[simplex::test]
fn assert_eq_1_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq1).equal().run(&context)
}

#[simplex::test]
fn assert_eq_1_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq1).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_8_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq8).equal().run(&context)
}

#[simplex::test]
fn assert_eq_8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq8).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_16_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq16).equal().run(&context)
}

#[simplex::test]
fn assert_eq_16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq16).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_32_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq32).equal().run(&context)
}

#[simplex::test]
fn assert_eq_32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq32).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_64_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq64).equal().run(&context)
}

#[simplex::test]
fn assert_eq_64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq64).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_128_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq128).equal().run(&context)
}

#[simplex::test]
fn assert_eq_128_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq128).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_256_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq256).equal().run(&context)
}

#[simplex::test]
fn assert_eq_256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEq256).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_eq_bool_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEqBool).equal().run(&context)
}

#[simplex::test]
fn assert_eq_bool_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertEqBool).expecting(&context, Expect::AssertFailed)
}

// assert_none: happy = None arg, unhappy = Some arg
#[simplex::test]
fn assert_none_1_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone1).none().run(&context)
}

#[simplex::test]
fn assert_none_1_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone1).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_none_8_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone8).none().run(&context)
}

#[simplex::test]
fn assert_none_8_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone8).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_none_16_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone16).none().run(&context)
}

#[simplex::test]
fn assert_none_16_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone16).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_none_32_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone32).none().run(&context)
}

#[simplex::test]
fn assert_none_32_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone32).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_none_64_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone64).none().run(&context)
}

#[simplex::test]
fn assert_none_64_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone64).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_none_128_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone128).none().run(&context)
}

#[simplex::test]
fn assert_none_128_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone128).expecting(&context, Expect::AssertFailed)
}

#[simplex::test]
fn assert_none_256_happy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone256).none().run(&context)
}

#[simplex::test]
fn assert_none_256_unhappy_path(context: simplex::TestContext) -> anyhow::Result<()> {
    case(AssertNone256).expecting(&context, Expect::AssertFailed)
}

mod asserts_test_fuzz {
    use super::*;

    use std::fmt::Debug;

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

    type AssertsFuzzEngineBuilder =
        FuzzEngineBuilder<AssertsTestProgram, AssertsTestArguments, AssertsTestWitness>;

    fn arb_u1() -> impl Strategy<Value = u8> {
        any::<bool>().prop_map(u8::from)
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

    fn arb_u256() -> impl Strategy<Value = [u8; 32]> {
        any::<[u8; 32]>()
    }

    type ArgumentFields<T> = fn(&mut AssertsTestWitness) -> (&mut Option<T>, &mut Option<T>);

    fn equal_values_strategy<T, V>(
        function_index: u8,
        values: impl Strategy<Value = T> + 'static,
        fields: ArgumentFields<V>,
    ) -> BoxedStrategy<AssertsTestWitness>
    where
        T: Clone + Debug + Into<V> + 'static,
        V: 'static,
    {
        values
            .prop_map(move |value| {
                let mut witness = default_witness(function_index);
                let first: V = value.clone().into();
                let second: V = value.into();

                Case::set_arguments(fields(&mut witness), first, second);
                witness
            })
            .boxed()
    }

    fn distinct_values_strategy<T, V>(
        function_index: u8,
        values: impl Strategy<Value = T> + 'static,
        fields: ArgumentFields<V>,
    ) -> BoxedStrategy<AssertsTestWitness>
    where
        T: PartialEq + Debug + Into<V> + 'static,
        V: 'static,
    {
        let values = values.boxed();

        (values.clone(), values)
            .prop_filter("assert_eq arguments must be distinct", |(first, second)| {
                first != second
            })
            .prop_map(move |(first, second)| {
                let mut witness = default_witness(function_index);
                let first: V = first.into();
                let second: V = second.into();

                Case::set_arguments(fields(&mut witness), first, second);
                witness
            })
            .boxed()
    }

    fn none_strategy<T: Default + Debug + 'static>(
        none: bool,
        function_index: u8,
        values: impl Strategy<Value = T> + 'static,
        fields: ArgumentFields<T>,
    ) -> BoxedStrategy<AssertsTestWitness> {
        values
            .prop_map(move |value| {
                let mut witness = default_witness(function_index);

                let (first, second) = if none {
                    (None, Some(value))
                } else {
                    (Some(value), Some(T::default()))
                };

                Case::set_arguments(fields(&mut witness), first, second);
                witness
            })
            .boxed()
    }

    impl Case {
        fn set_arguments<T>(
            fields: (&mut Option<T>, &mut Option<T>),
            first: impl Into<Option<T>>,
            second: impl Into<Option<T>>,
        ) {
            *fields.0 = first.into();
            *fields.1 = second.into();
        }

        fn u1_arguments(witness: &mut AssertsTestWitness) -> (&mut Option<u8>, &mut Option<u8>) {
            (&mut witness.first_arg_u1, &mut witness.second_arg_u1)
        }

        fn u8_arguments(witness: &mut AssertsTestWitness) -> (&mut Option<u8>, &mut Option<u8>) {
            (&mut witness.first_arg_u8, &mut witness.second_arg_u8)
        }

        fn u16_arguments(witness: &mut AssertsTestWitness) -> (&mut Option<u16>, &mut Option<u16>) {
            (&mut witness.first_arg_u16, &mut witness.second_arg_u16)
        }

        fn u32_arguments(witness: &mut AssertsTestWitness) -> (&mut Option<u32>, &mut Option<u32>) {
            (&mut witness.first_arg_u32, &mut witness.second_arg_u32)
        }

        fn u64_arguments(witness: &mut AssertsTestWitness) -> (&mut Option<u64>, &mut Option<u64>) {
            (&mut witness.first_arg_u64, &mut witness.second_arg_u64)
        }

        fn u128_arguments(
            witness: &mut AssertsTestWitness,
        ) -> (&mut Option<u128>, &mut Option<u128>) {
            (&mut witness.first_arg_u128, &mut witness.second_arg_u128)
        }

        fn u256_arguments(
            witness: &mut AssertsTestWitness,
        ) -> (&mut Option<[u8; 32]>, &mut Option<[u8; 32]>) {
            (&mut witness.first_arg_u256, &mut witness.second_arg_u256)
        }

        fn strategy(self) -> BoxedStrategy<AssertsTestWitness> {
            match (self.function, self.same) {
                (
                    AssertNone1 | AssertNone8 | AssertNone16 | AssertNone32 | AssertNone64
                    | AssertNone128 | AssertNone256,
                    _,
                ) => self.strategy_with_none_values(),
                (_, true) => self.strategy_with_same_values(),
                (_, false) => self.strategy_with_distinct_values(),
            }
        }

        fn strategy_with_none_values(&self) -> BoxedStrategy<AssertsTestWitness> {
            let function_index = self.function as u8;

            match self.function {
                AssertNone1 => {
                    none_strategy(self.none, function_index, arb_u1(), Self::u1_arguments)
                }
                AssertNone8 => {
                    none_strategy(self.none, function_index, arb_u8(), Self::u8_arguments)
                }
                AssertNone16 => {
                    none_strategy(self.none, function_index, arb_u16(), Self::u16_arguments)
                }
                AssertNone32 => {
                    none_strategy(self.none, function_index, arb_u32(), Self::u32_arguments)
                }
                AssertNone64 => {
                    none_strategy(self.none, function_index, arb_u64(), Self::u64_arguments)
                }
                AssertNone128 => {
                    none_strategy(self.none, function_index, arb_u128(), Self::u128_arguments)
                }
                AssertNone256 => {
                    none_strategy(self.none, function_index, arb_u256(), Self::u256_arguments)
                }
                _ => unreachable!("assert_eq cases aren't handled"),
            }
        }

        fn strategy_with_same_values(&self) -> BoxedStrategy<AssertsTestWitness> {
            let function_index = self.function as u8;

            match self.function {
                AssertEq1 => equal_values_strategy(function_index, arb_u1(), Self::u1_arguments),
                AssertEq8 => equal_values_strategy(function_index, arb_u8(), Self::u8_arguments),
                AssertEq16 => equal_values_strategy(function_index, arb_u16(), Self::u16_arguments),
                AssertEq32 => equal_values_strategy(function_index, arb_u32(), Self::u32_arguments),
                AssertEq64 => equal_values_strategy(function_index, arb_u64(), Self::u64_arguments),
                AssertEq128 => {
                    equal_values_strategy(function_index, arb_u128(), Self::u128_arguments)
                }
                AssertEq256 => {
                    equal_values_strategy(function_index, arb_u256(), Self::u256_arguments)
                }
                AssertEqBool => {
                    equal_values_strategy(function_index, arb_bool(), Self::u1_arguments)
                }
                _ => unreachable!("assert_none cases aren't handled"),
            }
        }

        fn strategy_with_distinct_values(&self) -> BoxedStrategy<AssertsTestWitness> {
            let function_index = self.function as u8;

            match self.function {
                AssertEq1 => distinct_values_strategy(function_index, arb_u1(), Self::u1_arguments),
                AssertEq8 => distinct_values_strategy(function_index, arb_u8(), Self::u8_arguments),
                AssertEq16 => {
                    distinct_values_strategy(function_index, arb_u16(), Self::u16_arguments)
                }
                AssertEq32 => {
                    distinct_values_strategy(function_index, arb_u32(), Self::u32_arguments)
                }
                AssertEq64 => {
                    distinct_values_strategy(function_index, arb_u64(), Self::u64_arguments)
                }
                AssertEq128 => {
                    distinct_values_strategy(function_index, arb_u128(), Self::u128_arguments)
                }
                AssertEq256 => {
                    distinct_values_strategy(function_index, arb_u256(), Self::u256_arguments)
                }
                AssertEqBool => {
                    distinct_values_strategy(function_index, arb_bool(), Self::u1_arguments)
                }
                _ => unreachable!("assert_none cases aren't handled"),
            }
        }
    }

    struct CaseFuzz {
        case: Case,
        builder: AssertsFuzzEngineBuilder,
        test_name: &'static str,
        expect: Expect,
    }

    fn case_fuzz(
        function: FunctionToTest,
        builder: AssertsFuzzEngineBuilder,
        test_name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            builder,
            test_name,
            expect: Expect::Ok,
        }
    }

    impl CaseFuzz {
        fn equal(mut self) -> Self {
            self.case = self.case.equal();
            self
        }

        fn none(mut self) -> Self {
            self.case = self.case.none();
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
            let strategy = self
                .case
                .strategy()
                .prop_map(|witness| {
                    let arguments: Arguments = AssertsTestArguments {}.into();
                    let witness: WitnessValues = witness.into();

                    (arguments, witness)
                })
                .boxed();

            let strategy =
                FuzzStrategyBuilder::<AssertsTestArguments, AssertsTestWitness, _>::new()
                    .with_custom_strategy(strategy)
                    .build();
            let transaction_builder =
                FinalTransactionBuilder::new(CaseFuzz::build_initial_tx(), [PROGRAM_TARGET])?;

            self.builder
                .build(strategy, transaction_builder)
                .run_with_check(FuzzExecutionCheck::new(self.test_name, self.expect));

            Ok(())
        }
    }

    #[simplex::fuzz]
    fn assert_eq_1_happy_path(fuzz_engine_builder: AssertsFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(AssertEq1, fuzz_engine_builder, "assert_eq_1_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_1_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq1, fuzz_engine_builder, "assert_eq_1_unhappy_path")
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_8_happy_path(fuzz_engine_builder: AssertsFuzzEngineBuilder) -> anyhow::Result<()> {
        case_fuzz(AssertEq8, fuzz_engine_builder, "assert_eq_8_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_8_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq8, fuzz_engine_builder, "assert_eq_8_unhappy_path")
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_16_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq16, fuzz_engine_builder, "assert_eq_16_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_16_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq16, fuzz_engine_builder, "assert_eq_16_unhappy_path")
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_32_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq32, fuzz_engine_builder, "assert_eq_32_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_32_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq32, fuzz_engine_builder, "assert_eq_32_unhappy_path")
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_64_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq64, fuzz_engine_builder, "assert_eq_64_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_64_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq64, fuzz_engine_builder, "assert_eq_64_unhappy_path")
            .expect(Expect::AssertFailed)
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_128_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq128, fuzz_engine_builder, "assert_eq_128_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_128_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertEq128,
            fuzz_engine_builder,
            "assert_eq_128_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_eq_256_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertEq256, fuzz_engine_builder, "assert_eq_256_happy_path")
            .equal()
            .run()
    }

    #[simplex::fuzz]
    fn assert_eq_256_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertEq256,
            fuzz_engine_builder,
            "assert_eq_256_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_eq_bool_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertEqBool,
            fuzz_engine_builder,
            "assert_eq_bool_happy_path",
        )
        .equal()
        .run()
    }

    #[simplex::fuzz]
    fn assert_eq_bool_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertEqBool,
            fuzz_engine_builder,
            "assert_eq_bool_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_1_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertNone1, fuzz_engine_builder, "assert_none_1_happy_path")
            .none()
            .run()
    }

    #[simplex::fuzz]
    fn assert_none_1_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone1,
            fuzz_engine_builder,
            "assert_none_1_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_8_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(AssertNone8, fuzz_engine_builder, "assert_none_8_happy_path")
            .none()
            .run()
    }

    #[simplex::fuzz]
    fn assert_none_8_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone8,
            fuzz_engine_builder,
            "assert_none_8_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_16_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone16,
            fuzz_engine_builder,
            "assert_none_16_happy_path",
        )
        .none()
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_16_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone16,
            fuzz_engine_builder,
            "assert_none_16_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_32_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone32,
            fuzz_engine_builder,
            "assert_none_32_happy_path",
        )
        .none()
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_32_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone32,
            fuzz_engine_builder,
            "assert_none_32_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_64_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone64,
            fuzz_engine_builder,
            "assert_none_64_happy_path",
        )
        .none()
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_64_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone64,
            fuzz_engine_builder,
            "assert_none_64_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_128_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone128,
            fuzz_engine_builder,
            "assert_none_128_happy_path",
        )
        .none()
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_128_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone128,
            fuzz_engine_builder,
            "assert_none_128_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_256_happy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone256,
            fuzz_engine_builder,
            "assert_none_256_happy_path",
        )
        .none()
        .run()
    }

    #[simplex::fuzz]
    fn assert_none_256_unhappy_path(
        fuzz_engine_builder: AssertsFuzzEngineBuilder,
    ) -> anyhow::Result<()> {
        case_fuzz(
            AssertNone256,
            fuzz_engine_builder,
            "assert_none_256_unhappy_path",
        )
        .expect(Expect::AssertFailed)
        .run()
    }
}
