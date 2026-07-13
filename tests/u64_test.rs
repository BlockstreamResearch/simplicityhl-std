mod common;

use common::uint::TestUint;

use simplicityhl_std::artifacts::u64_test::U64TestProgram;
use simplicityhl_std::artifacts::u64_test::derived_u64_test::{U64TestArguments, U64TestWitness};

use crate::common::uint::CUSTOM_BASE;

// The only per-width code for the common operations.
impl TestUint for u64 {
    type Program = U64TestProgram;
    type Witness = U64TestWitness;

    const ZERO: u64 = 0;
    const ONE: u64 = 1;
    const MAX: u64 = u64::MAX;
    const HALF_MAX: u64 = u64::MAX / 2;
    const MUL_BOUND: u64 = 1 << 32; // 2^(64/2)

    fn program() -> U64TestProgram {
        U64TestProgram::new(U64TestArguments {})
    }

    fn witness(op: u8, a: u64, b: u64, expected: Option<u64>) -> U64TestWitness {
        U64TestWitness {
            function_index: op,
            first_arg: a,
            second_arg: b,
            expected,
            ..Default::default()
        }
    }
}

enum FunctionToTest {
    U64Widen,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8 + CUSTOM_BASE
}

mod u64_tests {
    use rand::Rng as _;

    use crate::{
        FunctionToTest::U64Widen,
        common::core::{Expect, run},
    };

    use super::*;

    // Stamps the 16 `#[simplex::test]` entry points for u64. Logic lives in common::uint.
    uint_tests!(u64);

    #[simplex::test]
    fn u64_widen(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = rand::thread_rng().gen_range(0..=u64::MAX);

        let mut expected_widen: [u8; 32] = [0u8; 32];
        expected_widen[24..32].copy_from_slice(&a.to_be_bytes());

        run(
            &context,
            <u64 as TestUint>::program(),
            U64TestWitness {
                function_index: op(U64Widen),
                first_arg: a,
                expected_widen,
                ..Default::default()
            },
            Expect::Ok,
        )
    }
}
