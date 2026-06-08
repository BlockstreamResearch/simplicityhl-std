use rand::Rng;

pub enum IfTestOverflow {
    NotOverflow,
    Overflow,
}

pub fn cast_to_bool(flag: u8) -> bool {
    flag == 1
}

pub fn generate_uints_for_test(if_same: bool, min_val: u128, max_val: u128) -> (u128, u128) {
    let some_u = rand::thread_rng().gen_range(min_val..=max_val);

    if if_same {
        (some_u, some_u)
    } else {
        let mut other_u = rand::thread_rng().gen_range(min_val..=max_val);

        while other_u == some_u {
            other_u = rand::thread_rng().gen_range(min_val..=max_val);
        }

        (some_u, other_u)
    }
}
