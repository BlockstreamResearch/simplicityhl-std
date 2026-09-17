use rand::Rng;

use crate::common::core::{Expect, run};

use simplicityhl_std::artifacts::tests::asserts::AssertsProgram as AssertsTestProgram;
use simplicityhl_std::artifacts::tests::asserts::derived_asserts::{
    AssertsArguments as AssertsTestArguments, AssertsWitness as AssertsTestWitness,
};

// Dispatch indices which must match the `is_selected(N, ..)` arms in
// simf/tests/asserts.simf.
use FunctionToTest::*;

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

/// Builds the witness for one assert call. `same` controls the two `assert_eq`
/// args; `none` makes the single `assert_none` arg `None`.
fn build_witness(function: FunctionToTest, same: bool, none: bool) -> AssertsTestWitness {
    let mut witness = AssertsTestWitness {
        function_index: 0,
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
    };

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

    witness.function_index = function as u8;
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
