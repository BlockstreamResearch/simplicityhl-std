use crate::common::uint::TestUint;

use simplicityhl_std::artifacts::tests::u16::math::MathProgram as U16MathTestProgram;
use simplicityhl_std::artifacts::tests::u16::math::derived_math::{
    MathArguments as U16MathTestArguments, MathWitness as U16MathTestWitness,
};

// The only per-width code for the common operations.
impl TestUint for u16 {
    type Program = U16MathTestProgram;
    type Witness = U16MathTestWitness;

    const ZERO: u16 = 0;
    const ONE: u16 = 1;
    const MAX: u16 = u16::MAX;
    const HALF_MAX: u16 = u16::MAX / 2;
    const MUL_BOUND: u16 = 1 << 8; // 2^(16/2)

    fn program() -> U16MathTestProgram {
        U16MathTestProgram::new(U16MathTestArguments {})
    }

    fn witness(op: u8, a: u16, b: u16, expected: Option<u16>) -> U16MathTestWitness {
        U16MathTestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
        }
    }
}

// Stamps the 22 `#[simplex::test]` entry points for u16. Logic lives in common::uint.
crate::uint_tests!(u16);

mod math_tests_fuzz {
    use super::*;

    use crate::common::uint_fuzz::TestUintFuzz;
    use simplex::fuzz::FuzzEngineBuilder;
    use simplex::fuzz::proptest::prelude::any;
    use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};

    type U16MathFuzzEngineBuilder =
        FuzzEngineBuilder<U16MathTestProgram, U16MathTestArguments, U16MathTestWitness>;

    impl TestUintFuzz for u16 {
        type Arguments = U16MathTestArguments;

        fn arguments() -> Self::Arguments {
            U16MathTestArguments {}
        }

        fn arb_any() -> BoxedStrategy<Self> {
            any::<u16>().boxed()
        }

        fn arb_non_zero() -> BoxedStrategy<Self> {
            any::<u16>().prop_map(|value| value.max(1)).boxed()
        }

        fn arb_fitting(low: Self, high: Self) -> BoxedStrategy<Self> {
            assert!(low <= high);
            (low..=high).boxed()
        }
    }

    crate::uint_fuzz_tests!(u16, U16MathFuzzEngineBuilder);
}
