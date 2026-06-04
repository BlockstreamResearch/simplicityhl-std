pub enum IfTestOverflow {
    NotOverflow,
    Overflow,
}

pub fn cast_to_bool(if_overflow: IfTestOverflow) -> bool {
    if_overflow as u8 == 1
}
