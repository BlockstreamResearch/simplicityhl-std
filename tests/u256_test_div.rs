mod common;

use primitive_types::U256;

use crate::common::helper::generate_u256;
use common::core::{Expect, run};

use simplicityhl_std::artifacts::u256_test_div::U256TestDivProgram;
use simplicityhl_std::artifacts::u256_test_div::derived_u256_test_div::{
    U256TestDivArguments, U256TestDivWitness,
};

enum FunctionToTest {
    DivMod256_64,
    DivMod256_128,
    DivMod256,
    Div256,
}

#[inline]
fn op(o: FunctionToTest) -> u8 {
    o as u8
}

const DEFAULT_EXPECTED: [u8; 32] = [0; 32];

fn program() -> U256TestDivProgram {
    U256TestDivProgram::new(&U256TestDivArguments {})
}

fn build_witness(
    function: u8,
    a: [u8; 32],
    b: [u8; 32],
    expected: Option<[u8; 32]>,
    second_expected: [u8; 32],
) -> U256TestDivWitness {
    U256TestDivWitness {
        function_index: function,
        first_arg: a,
        second_arg: b,
        expected,
        second_expected,
    }
}

mod u256_tests_arithmetic {
    use super::*;

    #[simplex::test]
    fn test_div_mod_256_64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u64::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_64),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_64_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_64),
                a.to_big_endian(),
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::from(u64::MAX) + 1, U256::from(u128::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128_b_fits_into_u64(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::one(), U256::from(u64::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256_128),
                a.to_big_endian(),
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }

    #[simplex::test]
    fn test_div_mod_256_128_a_eq_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = (generate_u256(U256::one(), U256::from(u128::MAX))).to_big_endian();

        let q = U256::one().to_big_endian();
        let r = U256::zero().to_big_endian();

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::DivMod256_128), a, a, Some(q), r),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_a_less_than_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX - 1);
        let b = generate_u256(a + 1, U256::MAX);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_div_128(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let a = generate_u256(b, U256::from(u128::MAX));

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_q_is_1(context: simplex::TestContext) -> anyhow::Result<()> {
        // case where a >= b and a_high = b_high != 0
        let b_low = generate_u256(U256::zero(), U256::from(u128::MAX));
        let a_low = generate_u256(b_low, U256::from(u128::MAX));
        let high = generate_u256(U256::one(), U256::from(u128::MAX));

        let a = (high << 128) | (a_low);
        let b = (high << 128) | (b_low);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_b_fits_into_u128(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = generate_u256(U256::one(), U256::from(u128::MAX));
        let a = generate_u256(U256::from(u128::MAX) + 1, U256::MAX);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_b_is_u256(context: simplex::TestContext) -> anyhow::Result<()> {
        let b = generate_u256(U256::one(), U256::MAX - 1);
        let a = generate_u256(b + 1, U256::MAX);

        let q = (a / b).to_big_endian();
        let r = (a % b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q),
                r,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_a_equal_b(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::one(), U256::MAX).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a,
                a,
                Some(U256::one().to_big_endian()),
                [0; 32],
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_equal_high_words_max_low_diff(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let high = generate_u256(U256::one(), U256::from(u128::MAX));

        let a = ((high << 128) | (U256::from(u128::MAX))).to_big_endian();
        let b = (high << 128).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a,
                b,
                Some(U256::one().to_big_endian()),
                U256::from(u128::MAX).to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_eq_high_words_a_less_than_b(
        context: simplex::TestContext,
    ) -> anyhow::Result<()> {
        let high = generate_u256(U256::one(), U256::from(u128::MAX));

        let a = (high << 128).to_big_endian();
        let b = ((high << 128) | (U256::from(u128::MAX))).to_big_endian();

        run(
            &context,
            program(),
            build_witness(op(FunctionToTest::DivMod256), a, b, Some([0; 32]), a),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_mod_256_edge_case(context: simplex::TestContext) -> anyhow::Result<()> {
        let a: U256 = U256::from(2).pow(U256::from(255));
        let b = U256::from(2).pow(U256::from(127)) + U256::from(2).pow(U256::from(64)) - 1;

        let (q, r) = a.div_mod(b);

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::DivMod256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(q.to_big_endian()),
                r.to_big_endian(),
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_256(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = generate_u256(U256::one(), U256::MAX);
        let result = (a / b).to_big_endian();

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Div256),
                a.to_big_endian(),
                b.to_big_endian(),
                Some(result),
                DEFAULT_EXPECTED,
            ),
            Expect::Ok,
        )
    }

    #[simplex::test]
    fn u256_test_div_256_overflow(context: simplex::TestContext) -> anyhow::Result<()> {
        let a = generate_u256(U256::zero(), U256::MAX);
        let b = [0; 32];

        run(
            &context,
            program(),
            build_witness(
                op(FunctionToTest::Div256),
                a.to_big_endian(),
                b,
                Some(DEFAULT_EXPECTED),
                DEFAULT_EXPECTED,
            ),
            Expect::AssertFailed,
        )
    }
}
