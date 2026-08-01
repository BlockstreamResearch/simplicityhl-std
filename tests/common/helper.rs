use primitive_types::U256;
use rand::Rng;

#[allow(dead_code)]
pub const DEFAULT_BOOL: bool = false;

#[allow(dead_code)]
pub fn generate_u256(lower_bound: U256, upper_bound: U256) -> U256 {
    assert!(
        lower_bound <= upper_bound,
        "Error: lower bound is greater than upper bound"
    );

    let (a_high, a_low): (u128, u128) = if lower_bound > U256::from(u128::MAX) {
        (
            rand::thread_rng()
                .gen_range((lower_bound >> 128).as_u128()..=(upper_bound >> 128).as_u128()),
            rand::thread_rng().gen_range(0..=u128::MAX),
        )
    } else if upper_bound > U256::from(u128::MAX) {
        (
            rand::thread_rng().gen_range(0_u128..=(upper_bound >> 128).as_u128()),
            rand::thread_rng().gen_range(lower_bound.as_u128()..=u128::MAX),
        )
    } else {
        (
            0,
            rand::thread_rng().gen_range(lower_bound.as_u128()..=upper_bound.as_u128()),
        )
    };

    (U256::from(a_high) << 128) | U256::from(a_low)
}
