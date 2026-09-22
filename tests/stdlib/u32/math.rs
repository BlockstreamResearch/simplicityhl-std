use crate::common::uint::TestUint;

use simplicityhl_std::artifacts::tests::u32::math::MathProgram as U32MathTestProgram;
use simplicityhl_std::artifacts::tests::u32::math::derived_math::{
    MathArguments as U32MathTestArguments, MathWitness as U32MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u32 {
    type Program = U32MathTestProgram;
    type Witness = U32MathTestWitness;

    const ZERO: u32 = 0;
    const ONE: u32 = 1;
    const MAX: u32 = u32::MAX;
    const HALF_MAX: u32 = u32::MAX / 2;
    const MUL_BOUND: u32 = 1 << 16; // 2^(32/2)

    fn program() -> U32MathTestProgram {
        U32MathTestProgram::new(U32MathTestArguments {})
    }

    fn witness(op: u8, a: u32, b: u32, expected: Option<u32>) -> U32MathTestWitness {
        U32MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

// Stamps the 22 `#[simplex::test]` entry points for u32. Logic lives in common::uint.
crate::uint_tests!(u32);

mod math_tests_fuzz {
    use super::*;

    use crate::common::uint_fuzz::TestUintFuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};

    type U32MathFuzzEngineBuilder =
        FuzzEngineBuilder<U32MathTestProgram, U32MathTestArguments, U32MathTestWitness>;

    impl TestUintFuzz for u32 {
        type Arguments = U32MathTestArguments;

        fn arguments() -> Self::Arguments {
            U32MathTestArguments {}
        }

        fn arb_any() -> BoxedStrategy<Self> {
            any::<u32>().boxed()
        }

        fn arb_non_zero() -> BoxedStrategy<Self> {
            any::<u32>().prop_map(|value| value.max(1)).boxed()
        }

        fn arb_fitting(low: Self, high: Self) -> BoxedStrategy<Self> {
            assert!(low <= high);
            (low..=high).boxed()
        }
    }

    crate::uint_fuzz_tests!(u32, U32MathFuzzEngineBuilder);
}
