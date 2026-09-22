use crate::common::uint::TestUint;

use simplicityhl_std::artifacts::tests::u8::math::MathProgram as U8MathTestProgram;
use simplicityhl_std::artifacts::tests::u8::math::derived_math::{
    MathArguments as U8MathTestArguments, MathWitness as U8MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u8 {
    type Program = U8MathTestProgram;
    type Witness = U8MathTestWitness;

    const ZERO: u8 = 0;
    const ONE: u8 = 1;
    const MAX: u8 = u8::MAX;
    const HALF_MAX: u8 = u8::MAX / 2;
    const MUL_BOUND: u8 = 1 << 4; // 2^(8/2)

    fn program() -> U8MathTestProgram {
        U8MathTestProgram::new(U8MathTestArguments {})
    }

    fn witness(op: u8, a: u8, b: u8, expected: Option<u8>) -> U8MathTestWitness {
        U8MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

// Stamps the 22 `#[simplex::test]` entry points for u8. Logic lives in common::uint.
crate::uint_tests!(u8);

mod math_tests_fuzz {
    use super::*;

    use crate::common::uint_fuzz::TestUintFuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};

    type U8MathFuzzEngineBuilder =
        FuzzEngineBuilder<U8MathTestProgram, U8MathTestArguments, U8MathTestWitness>;

    impl TestUintFuzz for u8 {
        type Arguments = U8MathTestArguments;

        fn arguments() -> Self::Arguments {
            U8MathTestArguments {}
        }

        fn arb_any() -> BoxedStrategy<Self> {
            any::<u8>().boxed()
        }

        fn arb_non_zero() -> BoxedStrategy<Self> {
            any::<u8>().prop_map(|value| value.max(1)).boxed()
        }

        fn arb_fitting(low: Self, high: Self) -> BoxedStrategy<Self> {
            assert!(low <= high);
            (low..=high).boxed()
        }
    }

    crate::uint_fuzz_tests!(u8, U8MathFuzzEngineBuilder);
}
