pub enum IfTestOverflow {
    NotOverflow,
    Overflow,
}

pub fn cast_to_bool(flag: u8) -> bool {
    flag == 1
}
