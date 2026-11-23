use std::fmt::Debug;

use simplex::fuzz::proptest::prelude::any;
use simplex::fuzz::proptest::strategy::{BoxedStrategy, Strategy};
use simplex::{FuzzMode, TestContext};

use crate::asserts::{case, case::U256};
use crate::common::fuzz::{accepts, rejects};

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

fn arb_u256() -> impl Strategy<Value = U256> {
    any::<U256>()
}

fn arb_distinct<T>(values: impl Strategy<Value = T> + 'static) -> impl Strategy<Value = (T, T)>
where
    T: PartialEq + Debug + 'static,
{
    let values: BoxedStrategy<T> = values.boxed();

    (values.clone(), values).prop_filter("distinct values", |(a, b)| a != b)
}

// 0. assert_eq_1
#[simplex::fuzz]
fn assert_eq_1_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u1(), |a| case::assert_eq_1(a, a));
}

#[simplex::fuzz]
fn assert_eq_1_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u1()), |(a, b)| {
        case::assert_eq_1(a, b)
    });
}

// 1. assert_eq_8
#[simplex::fuzz]
fn assert_eq_8_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u8(), |a| case::assert_eq_8(a, a));
}

#[simplex::fuzz]
fn assert_eq_8_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u8()), |(a, b)| {
        case::assert_eq_8(a, b)
    });
}

// 2. assert_eq_16
#[simplex::fuzz]
fn assert_eq_16_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u16(), |a| case::assert_eq_16(a, a));
}

#[simplex::fuzz]
fn assert_eq_16_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u16()), |(a, b)| {
        case::assert_eq_16(a, b)
    });
}

// 3. assert_eq_32
#[simplex::fuzz]
fn assert_eq_32_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u32(), |a| case::assert_eq_32(a, a));
}

#[simplex::fuzz]
fn assert_eq_32_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u32()), |(a, b)| {
        case::assert_eq_32(a, b)
    });
}

// 4. assert_eq_64
#[simplex::fuzz]
fn assert_eq_64_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u64(), |a| case::assert_eq_64(a, a));
}

#[simplex::fuzz]
fn assert_eq_64_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u64()), |(a, b)| {
        case::assert_eq_64(a, b)
    });
}

// 5. assert_eq_128
#[simplex::fuzz]
fn assert_eq_128_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u128(), |a| case::assert_eq_128(a, a));
}

#[simplex::fuzz]
fn assert_eq_128_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u128()), |(a, b)| {
        case::assert_eq_128(a, b)
    });
}

// 6. assert_eq_256
#[simplex::fuzz]
fn assert_eq_256_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_u256(), |a| case::assert_eq_256(a, a));
}

#[simplex::fuzz]
fn assert_eq_256_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_u256()), |(a, b)| {
        case::assert_eq_256(a, b)
    });
}

// 7. assert_eq_bool
#[simplex::fuzz]
fn assert_eq_bool_accepts_equal(context: TestContext<FuzzMode>) {
    accepts(&context, arb_bool(), |a| case::assert_eq_bool(a, a));
}

#[simplex::fuzz]
fn assert_eq_bool_rejects_distinct(context: TestContext<FuzzMode>) {
    rejects(&context, arb_distinct(arb_bool()), |(a, b)| {
        case::assert_eq_bool(a, b)
    });
}

// 8. assert_none_1
#[simplex::fuzz]
fn assert_none_1_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u1(), |a| case::assert_none_1(Some(a)));
}

// 9. assert_none_8
#[simplex::fuzz]
fn assert_none_8_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u8(), |a| case::assert_none_8(Some(a)));
}

// 10. assert_none_16
#[simplex::fuzz]
fn assert_none_16_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u16(), |a| case::assert_none_16(Some(a)));
}

// 11. assert_none_32
#[simplex::fuzz]
fn assert_none_32_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u32(), |a| case::assert_none_32(Some(a)));
}

// 12. assert_none_64
#[simplex::fuzz]
fn assert_none_64_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u64(), |a| case::assert_none_64(Some(a)));
}

// 13. assert_none_128
#[simplex::fuzz]
fn assert_none_128_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u128(), |a| case::assert_none_128(Some(a)));
}

// 14. assert_none_256
#[simplex::fuzz]
fn assert_none_256_rejects_some(context: TestContext<FuzzMode>) {
    rejects(&context, arb_u256(), |a| case::assert_none_256(Some(a)));
}
