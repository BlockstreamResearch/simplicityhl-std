use primitive_types::U256;

use crate::common::core::{Expect, run};
use crate::common::helper::generate_u256;

use simplicityhl_std::artifacts::tests::u256::math::add::AddProgram as U256TestAddProgram;
use simplicityhl_std::artifacts::tests::u256::math::add::derived_add::{
    AddArguments as U256TestAddArguments, AddWitness as U256TestAddWitness,
};

use FunctionToTest::*;

enum FunctionToTest {
    Add256Carry,
    Add256_128Carry,
    FullAdd256,
}

fn program() -> U256TestAddProgram {
    U256TestAddProgram::new(&U256TestAddArguments {})
}

/// One dispatch arm of the contract, plus the witness it reads.
struct Case {
    witness: U256TestAddWitness,
}

fn case(function: FunctionToTest) -> Case {
    Case {
        witness: U256TestAddWitness {
            function_index: function as u8,
            first_arg: [0; 32],
            second_arg: [0; 32],
            third_arg: false,
            expected: None,
            expected_bool: false,
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
    fn third_arg(mut self, c: bool) -> Self {
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
fn add_256_carry_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::zero(), U256::MAX / 2);
    let result = (a + b).to_big_endian();

    case(Add256Carry)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn add_256_carry_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::MAX);
    let result = (b - 1).to_big_endian();

    case(Add256Carry)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn add_256_128_carry_not_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let result = (a + b).to_big_endian();

    case(Add256_128Carry)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .run(&context)
}

#[simplex::test]
fn add_256_128_carry_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::from(u128::MAX));
    let result = (b - 1).to_big_endian();

    case(Add256_128Carry)
        .args(a.to_big_endian(), b.to_big_endian())
        .expect(result)
        .flag(true)
        .run(&context)
}

#[simplex::test]
fn full_add_256_not_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::zero(), U256::MAX / 2);

    let result = (a + b).to_big_endian();
    let result_carry = false;
    let carry_low = false;

    case(FullAdd256)
        .args(a.to_big_endian(), b.to_big_endian())
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

#[simplex::test]
fn full_add_256_overflow_carry_low_false(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::MAX);

    let result = (b - 1).to_big_endian();
    let result_carry = true;
    let carry_low = false;

    case(FullAdd256)
        .args(a.to_big_endian(), b.to_big_endian())
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

#[simplex::test]
fn full_add_256_not_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = generate_u256(U256::zero(), U256::MAX / 2);
    let b = generate_u256(U256::zero(), U256::MAX / 2);

    let result = (a + b + 1).to_big_endian();
    let result_carry = false;
    let carry_low = true;

    case(FullAdd256)
        .args(a.to_big_endian(), b.to_big_endian())
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

#[simplex::test]
fn full_add_256_overflow_carry_low_true(context: simplex::TestContext) -> anyhow::Result<()> {
    let a = U256::MAX;
    let b = generate_u256(U256::one(), U256::MAX).to_big_endian();

    let result = b;
    let result_carry = true;
    let carry_low = true;

    case(FullAdd256)
        .args(a.to_big_endian(), b)
        .third_arg(carry_low)
        .expect(result)
        .flag(result_carry)
        .run(&context)
}

mod add_tests_fuzz {
    use std::cmp::max;

    use super::*;

    use simplex::simplicityhl::{Arguments, WitnessValues};

    use simplex::fuzz::fuzz_check::{Expect, FuzzExecutionCheck};
    use simplex::fuzz::proptest::prelude::{Just, any};
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
    use simplex::{FuzzMode, TestContext};

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
        carry_in: bool,
        expected: U256,
        expected_bool: bool,
    }

    impl FuzzCase {
        fn new(first_arg: U256, second_arg: U256) -> Self {
            Self {
                first_arg,
                second_arg,
                carry_in: false,
                expected: U256::zero(),
                expected_bool: false,
            }
        }

        fn carry_in(mut self, carry_in: bool) -> Self {
            self.carry_in = carry_in;
            self
        }

        fn expect(mut self, expected: U256) -> Self {
            self.expected = expected;
            self
        }

        fn expect_bool(mut self, expected_bool: bool) -> Self {
            self.expected_bool = expected_bool;
            self
        }

        fn into_witness(self, witness: U256TestAddWitness) -> WitnessValues {
            Case { witness }
                .args(
                    self.first_arg.to_big_endian(),
                    self.second_arg.to_big_endian(),
                )
                .third_arg(self.carry_in)
                .expect(self.expected.to_big_endian())
                .flag(self.expected_bool)
                .witness
                .into()
        }
    }

    struct CaseFuzz {
        case: Case,
        context: TestContext<FuzzMode>,
        inputs: Option<BoxedStrategy<FuzzCase>>,
        test_name: &'static str,
    }

    fn case_fuzz(
        function: FunctionToTest,
        context: TestContext<FuzzMode>,
        test_name: &'static str,
    ) -> CaseFuzz {
        CaseFuzz {
            case: case(function),
            context,
            inputs: None,
            test_name,
        }
    }

    impl CaseFuzz {
        fn strategy(mut self, inputs: impl Strategy<Value = FuzzCase> + 'static) -> Self {
            self.inputs = Some(inputs.boxed());
            self
        }

        fn run(self) -> anyhow::Result<()> {
            let Case { witness } = self.case;
            let inputs = self.inputs.expect("a fuzz strategy must be specified");

            let strategy = inputs
                .prop_map(move |case| {
                    let arguments: Arguments = U256TestAddArguments {}.into();
                    let witness = case.into_witness(witness.clone());

                    (arguments, witness)
                })
                .boxed();

            self.context
                .engine::<U256TestAddProgram, U256TestAddArguments, U256TestAddWitness>()
                .with_custom_strategy(strategy)
                .with_custom_check(FuzzExecutionCheck::new(self.test_name, Expect::Ok))
                .run();

            Ok(())
        }
    }

    // add_256_carry_not_overflow - accepts custom inputs for the same operation.
    // Custom full-width a; 0<=b<=MAX-a guarantees no output carry.
    #[simplex::fuzz]
    fn add_256_not_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::zero(), U256::MAX - a);

                b.prop_map(move |b| FuzzCase::new(a, b).expect(a + b))
            })
        };

        case_fuzz(Add256Carry, context, "u256 add without overflow")
            .strategy(strategy)
            .run()
    }

    // add_256_carry_overflow - accepts custom inputs for the same operation.
    // Custom a>0; MAX-a+1<=b<=MAX guarantees overflow; checks wrapped result and carry.
    #[simplex::fuzz]
    fn add_256_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::MAX - a + U256::one(), U256::MAX);

                b.prop_map(move |b| {
                    let (expected, carry) = a.overflowing_add(b);

                    FuzzCase::new(a, b).expect(expected).expect_bool(carry)
                })
            })
        };

        case_fuzz(Add256Carry, context, "u256 add with overflow")
            .strategy(strategy)
            .run()
    }

    // add_256_128_carry_not_overflow - accepts custom inputs for the same operation.
    // Custom full-width a; 0<=b<=min(MAX-a,u128::MAX) guarantees a fitting sum.
    #[simplex::fuzz]
    fn add_256_128_not_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::zero(), (U256::MAX - a).min(u128::MAX.into()));

                b.prop_map(move |b| FuzzCase::new(a, b).expect(a + b))
            })
        };

        case_fuzz(Add256_128Carry, context, "u256 add u128 without overflow")
            .strategy(strategy)
            .run()
    }

    // add_256_128_carry_overflow - accepts custom inputs for the same operation.
    // MAX-u128::MAX+1<=a<=MAX; MAX-a+1<=b<=u128::MAX guarantees overflow.
    #[simplex::fuzz]
    fn add_256_128_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let max_u128 = U256::from(u128::MAX);
        let strategy = {
            let a = arb_u256_in_range(U256::MAX - max_u128 + U256::one(), U256::MAX);

            a.prop_flat_map(move |a| {
                let b = arb_u256_in_range(U256::MAX - a + U256::one(), max_u128);

                b.prop_map(move |b| {
                    let (expected, carry) = a.overflowing_add(b);

                    FuzzCase::new(a, b).expect(expected).expect_bool(carry)
                })
            })
        };

        case_fuzz(Add256_128Carry, context, "u256 add u128 with overflow")
            .strategy(strategy)
            .run()
    }

    // full_add_256_not_overflow_carry_low_false - accepts custom inputs for the same operation.
    // carry_in=false; 0<=a<=MAX and 0<=b<=MAX-a; expects sum=a+b and carry_out=false.
    #[simplex::fuzz]
    fn full_add_256_not_overflow_carry_in_false(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::zero(), U256::MAX - a);

                b.prop_map(move |b| {
                    FuzzCase::new(a, b)
                        .carry_in(false)
                        .expect(a + b)
                        .expect_bool(false)
                })
            })
        };

        case_fuzz(
            FullAdd256,
            context,
            "u256 full add without overflow and input carry false",
        )
        .strategy(strategy)
        .run()
    }

    // full_add_256_overflow_carry_low_false - accepts custom inputs for the same operation.
    // carry_in=false; a>0 and MAX-a+1<=b<=MAX; checks wrapped sum and carry_out=true.
    #[simplex::fuzz]
    fn full_add_256_overflow_carry_in_false(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_non_zero_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::MAX - a + U256::one(), U256::MAX);

                b.prop_map(move |b| {
                    let (expected, carry) = a.overflowing_add(b);

                    FuzzCase::new(a, b)
                        .carry_in(false)
                        .expect(expected)
                        .expect_bool(carry)
                })
            })
        };

        case_fuzz(
            FullAdd256,
            context,
            "u256 full add with overflow and input carry false",
        )
        .strategy(strategy)
        .run()
    }

    // full_add_256_not_overflow_carry_low_true - accepts custom inputs for the same operation.
    // carry_in=true; 0<=a<MAX and 0<=b<=MAX-1-a; expects a+b+1 and carry_out=false.
    #[simplex::fuzz]
    fn full_add_256_not_overflow_carry_in_true(
        context: TestContext<FuzzMode>,
    ) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256_in_range(U256::zero(), U256::MAX - U256::one());

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::zero(), U256::MAX - U256::one() - a);

                b.prop_map(move |b| {
                    FuzzCase::new(a, b)
                        .carry_in(true)
                        .expect(a + b + U256::one())
                        .expect_bool(false)
                })
            })
        };

        case_fuzz(
            FullAdd256,
            context,
            "u256 full add without overflow and input carry true",
        )
        .strategy(strategy)
        .run()
    }

    // full_add_256_overflow_carry_low_true - accepts custom inputs for the same operation.
    // carry_in=true; 0<=a<=MAX and MAX-a<=b<=MAX; checks wrapped a+b+1 and carry_out=true.
    #[simplex::fuzz]
    fn full_add_256_overflow_carry_in_true(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        let strategy = {
            let a = arb_u256();

            a.prop_flat_map(|a| {
                let b = arb_u256_in_range(U256::MAX - a, U256::MAX);

                b.prop_map(move |b| {
                    let (result, carry) = a.overflowing_add(b);
                    let (expected, carry_from_input) = result.overflowing_add(U256::one());

                    FuzzCase::new(a, b)
                        .carry_in(true)
                        .expect(expected)
                        .expect_bool(carry || carry_from_input)
                })
            })
        };

        case_fuzz(
            FullAdd256,
            context,
            "u256 full add with overflow and input carry true",
        )
        .strategy(strategy)
        .run()
    }

    // No direct simplex::test counterpart - new input-bit boundary.
    // New boundary: a=MAX, b=0, carry_in=true; only the input carry causes result=0 and
    // carry_out=true.
    #[simplex::fuzz]
    fn full_add_256_carry_in_causes_overflow(context: TestContext<FuzzMode>) -> anyhow::Result<()> {
        // MAX + 0 fits; only the input carry causes overflow.
        let strategy = Just(U256::MAX).prop_map(|a| {
            FuzzCase::new(a, U256::zero())
                .carry_in(true)
                .expect(U256::zero())
                .expect_bool(true)
        });

        case_fuzz(
            FullAdd256,
            context,
            "u256 input carry alone causes overflow",
        )
        .strategy(strategy)
        .run()
    }
}
