/// Big-endian 256-bit integer.
pub type U256 = [u8; 32];

pub type Uint = primitive_types::U256;

pub const fn u256(value: u128) -> U256 {
    let mut out = [0; 32];
    let bytes = value.to_be_bytes();
    let mut i = 0;

    while i < 16 {
        out[16 + i] = bytes[i];
        i += 1;
    }

    out
}

pub fn int(value: U256) -> Uint {
    Uint::from_big_endian(&value)
}

pub fn add_u256(a: U256, b: U256) -> U256 {
    int(a)
        .checked_add(int(b))
        .expect("a + b overflows 256 bits")
        .to_big_endian()
}

pub fn sub_u256(a: U256, b: U256) -> U256 {
    int(a)
        .checked_sub(int(b))
        .expect("a - b underflows")
        .to_big_endian()
}

/// Operands may be non-canonical, the result is canonical.
pub fn sub_mod(a: U256, b: U256, modulus: U256) -> U256 {
    let m = int(modulus);
    let (a, b) = (int(a) % m, int(b) % m);

    let difference = if a >= b { a - b } else { m - (b - a) };
    difference.to_big_endian()
}
