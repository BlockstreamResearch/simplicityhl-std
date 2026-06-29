use rand::Rng;

#[allow(dead_code)]
pub enum IfTestOverflow {
    NotOverflow,
    Overflow,
}

pub fn cast_to_bool(flag: u8) -> bool {
    flag == 1
}

#[allow(dead_code)]
pub fn generate_uints_in_one_range(if_same: bool, min_val: u128, max_val: u128) -> (u128, u128) {
    let some_u = rand::thread_rng().gen_range(min_val..=max_val);

    if if_same {
        return (some_u, some_u);
    }

    if min_val == max_val {
        panic!("Can not generate different values when range is one number");
    }

    let mut other_u = rand::thread_rng().gen_range(min_val..=max_val);

    while other_u == some_u {
        other_u = rand::thread_rng().gen_range(min_val..=max_val);
    }

    (some_u, other_u)
}
