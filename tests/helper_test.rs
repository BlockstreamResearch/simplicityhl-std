mod common;

use primitive_types::U256;

use crate::common::helper::generate_u256;

#[test]
fn generate_u256_respects_bounds() {
    let cases = [
        (U256::zero(), U256::MAX),
        (U256::one(), U256::from(u128::MAX)),
        (U256::from(u128::MAX) + 1, U256::MAX),
        (U256::from(u64::MAX) + 1, U256::from(u128::MAX)),
        (U256::MAX, U256::MAX),
    ];

    for (lo, hi) in cases {
        for _ in 0..10_000 {
            let v = generate_u256(lo, hi);
            assert!(lo <= v && v <= hi, "{v:#x} outside [{lo:#x}, {hi:#x}]");
        }
    }
}
