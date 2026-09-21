use crate::common::uint::TestUint;

use simplicityhl_std::artifacts::tests::u128::math::api::ApiProgram as U128MathTestProgram;
use simplicityhl_std::artifacts::tests::u128::math::api::derived_api::{
    ApiArguments as U128MathTestArguments, ApiWitness as U128MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u128 {
    type Program = U128MathTestProgram;
    type Witness = U128MathTestWitness;

    const ZERO: u128 = 0;
    const ONE: u128 = 1;
    const MAX: u128 = u128::MAX;
    const HALF_MAX: u128 = u128::MAX / 2;
    const MUL_BOUND: u128 = 1 << 64; // 2^(128/2)

    fn program() -> U128MathTestProgram {
        U128MathTestProgram::new(U128MathTestArguments {})
    }

    fn witness(op: u8, a: u128, b: u128, expected: Option<u128>) -> U128MathTestWitness {
        U128MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

// Stamps the 22 `#[simplex::test]` entry points for u128. Logic lives in common::uint.
crate::uint_tests!(u128);

mod math_api_tests_fuzz {
    use super::*;
    use crate::common::uint_fuzz::TestUintFuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};

    type Builder =
        FuzzEngineBuilder<U128MathTestProgram, U128MathTestArguments, U128MathTestWitness>;

    impl TestUintFuzz for u128 {
        type Arguments = U128MathTestArguments;

        fn arguments() -> Self::Arguments {
            U128MathTestArguments {}
        }

        fn arb_any() -> BoxedStrategy<Self> {
            any::<u128>().boxed()
        }

        fn arb_non_zero() -> BoxedStrategy<Self> {
            any::<u128>().prop_map(|value| value.max(1)).boxed()
        }

        fn arb_fitting(low: Self, high: Self) -> BoxedStrategy<Self> {
            assert!(low <= high);
            (low..=high).boxed()
        }
    }

    crate::uint_fuzz_tests!(u128, Builder);
}
