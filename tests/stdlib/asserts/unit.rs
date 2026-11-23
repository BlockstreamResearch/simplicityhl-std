use crate::asserts::case;
use crate::asserts::case::U256;
use crate::common::case::DispatchCase;
use crate::common::core::Expect::AssertFailed;

// 0. assert_eq_1
#[test]
fn assert_eq_1_accepts_equal() {
    case::assert_eq_1(0, 0).accepts();
    case::assert_eq_1(1, 1).accepts();
}

#[test]
fn assert_eq_1_rejects_distinct() {
    case::assert_eq_1(0, 1).rejects(AssertFailed);
    case::assert_eq_1(1, 0).rejects(AssertFailed);
}

// 1. assert_eq_8
#[test]
fn assert_eq_8_accepts_equal() {
    case::assert_eq_8(0, 0).accepts();
    case::assert_eq_8(0xA5, 0xA5).accepts();
    case::assert_eq_8(u8::MAX, u8::MAX).accepts();
}

#[test]
fn assert_eq_8_rejects_distinct() {
    case::assert_eq_8(0, 1).rejects(AssertFailed);
    case::assert_eq_8(u8::MAX, 0).rejects(AssertFailed);
}

// 2. assert_eq_16
#[test]
fn assert_eq_16_accepts_equal() {
    case::assert_eq_16(0, 0).accepts();
    case::assert_eq_16(0xA5A5, 0xA5A5).accepts();
    case::assert_eq_16(u16::MAX, u16::MAX).accepts();
}

#[test]
fn assert_eq_16_rejects_distinct() {
    case::assert_eq_16(0, 1).rejects(AssertFailed);
    case::assert_eq_16(0x00FF, 0x01FF).rejects(AssertFailed);
}

// 3. assert_eq_32
#[test]
fn assert_eq_32_accepts_equal() {
    case::assert_eq_32(0, 0).accepts();
    case::assert_eq_32(0xA5A5_A5A5, 0xA5A5_A5A5).accepts();
    case::assert_eq_32(u32::MAX, u32::MAX).accepts();
}

#[test]
fn assert_eq_32_rejects_distinct() {
    case::assert_eq_32(0, 1).rejects(AssertFailed);
    case::assert_eq_32(u32::MAX, u32::MAX >> 1).rejects(AssertFailed);
}

// 4. assert_eq_64
#[test]
fn assert_eq_64_accepts_equal() {
    case::assert_eq_64(0, 0).accepts();
    case::assert_eq_64(0xA5A5_A5A5_A5A5_A5A5, 0xA5A5_A5A5_A5A5_A5A5).accepts();
    case::assert_eq_64(u64::MAX, u64::MAX).accepts();
}

#[test]
fn assert_eq_64_rejects_distinct() {
    case::assert_eq_64(0, 1).rejects(AssertFailed);
    case::assert_eq_64(u64::MAX, u64::MAX >> 1).rejects(AssertFailed);
}

// 5. assert_eq_128
#[test]
fn assert_eq_128_accepts_equal() {
    case::assert_eq_128(0, 0).accepts();
    case::assert_eq_128(u128::MAX, u128::MAX).accepts();
}

#[test]
fn assert_eq_128_rejects_distinct() {
    case::assert_eq_128(0, 1).rejects(AssertFailed);
    case::assert_eq_128(u128::from(u64::MAX), u128::MAX).rejects(AssertFailed);
}

// 6. assert_eq_256
#[test]
fn assert_eq_256_accepts_equal() {
    let value: U256 = [0xA5; 32];

    case::assert_eq_256([0; 32], [0; 32]).accepts();
    case::assert_eq_256(value, value).accepts();
    case::assert_eq_256([u8::MAX; 32], [u8::MAX; 32]).accepts();
}

#[test]
fn assert_eq_256_rejects_distinct() {
    let mut first_byte_differs: U256 = [0; 32];
    first_byte_differs[0] = 1;

    let mut last_byte_differs: U256 = [0; 32];
    last_byte_differs[31] = 1;

    case::assert_eq_256([0; 32], first_byte_differs).rejects(AssertFailed);
    case::assert_eq_256([0; 32], last_byte_differs).rejects(AssertFailed);
}

// 7. assert_eq_bool
#[test]
fn assert_eq_bool_accepts_equal() {
    case::assert_eq_bool(false, false).accepts();
    case::assert_eq_bool(true, true).accepts();
}

#[test]
fn assert_eq_bool_rejects_distinct() {
    case::assert_eq_bool(false, true).rejects(AssertFailed);
    case::assert_eq_bool(true, false).rejects(AssertFailed);
}

// 8. assert_none_1
#[test]
fn assert_none_1_accepts_none() {
    case::assert_none_1(None).accepts();
}

#[test]
fn assert_none_1_rejects_some() {
    case::assert_none_1(Some(0)).rejects(AssertFailed);
    case::assert_none_1(Some(1)).rejects(AssertFailed);
}

// 9. assert_none_8
#[test]
fn assert_none_8_accepts_none() {
    case::assert_none_8(None).accepts();
}

#[test]
fn assert_none_8_rejects_some() {
    case::assert_none_8(Some(0)).rejects(AssertFailed);
    case::assert_none_8(Some(u8::MAX)).rejects(AssertFailed);
}

// 10. assert_none_16
#[test]
fn assert_none_16_accepts_none() {
    case::assert_none_16(None).accepts();
}

#[test]
fn assert_none_16_rejects_some() {
    case::assert_none_16(Some(0)).rejects(AssertFailed);
    case::assert_none_16(Some(u16::MAX)).rejects(AssertFailed);
}

// 11. assert_none_32
#[test]
fn assert_none_32_accepts_none() {
    case::assert_none_32(None).accepts();
}

#[test]
fn assert_none_32_rejects_some() {
    case::assert_none_32(Some(0)).rejects(AssertFailed);
    case::assert_none_32(Some(u32::MAX)).rejects(AssertFailed);
}

// 12. assert_none_64
#[test]
fn assert_none_64_accepts_none() {
    case::assert_none_64(None).accepts();
}

#[test]
fn assert_none_64_rejects_some() {
    case::assert_none_64(Some(0)).rejects(AssertFailed);
    case::assert_none_64(Some(u64::MAX)).rejects(AssertFailed);
}

// 13. assert_none_128
#[test]
fn assert_none_128_accepts_none() {
    case::assert_none_128(None).accepts();
}

#[test]
fn assert_none_128_rejects_some() {
    case::assert_none_128(Some(0)).rejects(AssertFailed);
    case::assert_none_128(Some(u128::MAX)).rejects(AssertFailed);
}

// 14. assert_none_256
#[test]
fn assert_none_256_accepts_none() {
    case::assert_none_256(None).accepts();
}

#[test]
fn assert_none_256_rejects_some() {
    case::assert_none_256(Some([0; 32])).rejects(AssertFailed);
    case::assert_none_256(Some([u8::MAX; 32])).rejects(AssertFailed);
}
