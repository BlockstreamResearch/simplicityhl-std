use primitive_types::{U256 as Uint, U512};
use secp256k1_zkp::{PublicKey, SECP256K1, SecretKey, constants};

/// Big-endian 256-bit integer.
pub type U256 = [u8; 32];
/// Field element, mod p.
pub type Fe = U256;
/// Scalar, mod the group order n.
pub type Scalar = U256;
/// Affine point (x, y).
pub type Ge = (Fe, Fe);
/// Jacobian point ((X, Y), Z), representing affine (X/Z², Y/Z³).
pub type Gej = (Ge, Fe);
/// Compressed point (y parity, x).
pub type Point = (u8, Fe);

pub const ZERO: U256 = [0; 32];
pub const ONE: U256 = constants::ONE;

pub const SECP_P: U256 = constants::FIELD_SIZE;
pub const SECP_N: U256 = constants::CURVE_ORDER;

pub const INFINITY: Gej = ((ZERO, ZERO), ZERO);

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

fn int(value: U256) -> Uint {
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

pub fn fe_mul(a: Fe, b: Fe) -> Fe {
    let product = int(a).full_mul(int(b)) % U512::from(int(SECP_P));

    Uint::try_from(product)
        .expect("reduced mod p")
        .to_big_endian()
}

pub fn pk_to_ge(pk: &PublicKey) -> Ge {
    let serialized = pk.serialize_uncompressed(); // 0x04 || x(32) || y(32)

    (
        serialized[1..33].try_into().unwrap(),
        serialized[33..65].try_into().unwrap(),
    )
}

pub fn ge_to_pk((x, y): Ge) -> PublicKey {
    let mut serialized = [0x04; 65];
    serialized[1..33].copy_from_slice(&x);
    serialized[33..65].copy_from_slice(&y);

    PublicKey::from_slice(&serialized).expect("on-curve point")
}

pub fn decompress((parity, x): Point) -> Option<Ge> {
    let mut serialized = [0x02 | (parity & 1); 33];
    serialized[1..].copy_from_slice(&x);

    PublicKey::from_slice(&serialized)
        .ok()
        .map(|pk| pk_to_ge(&pk))
}

pub fn compress((x, y): Ge) -> Point {
    (y[31] & 1, x)
}

pub fn ge_to_gej(ge: Ge) -> Gej {
    (ge, ONE)
}

pub fn ge_negate((x, y): Ge) -> Ge {
    (x, sub_mod(ZERO, y, SECP_P))
}

/// `p - q`, `None` for the point at infinity (`p == q`).
pub fn ge_sub(p: Ge, q: Ge) -> Option<Ge> {
    ge_to_pk(p)
        .combine(&ge_to_pk(q).negate(SECP256K1))
        .ok()
        .map(|difference| pk_to_ge(&difference))
}

/// `(λ²X, λ³Y, λ)`: the same point as `(X, Y, 1)` for any non-zero λ.
pub fn rescale((x, y): Ge, lambda: Fe) -> Gej {
    let lambda_squared = fe_mul(lambda, lambda);
    let lambda_cubed = fe_mul(lambda_squared, lambda);

    ((fe_mul(x, lambda_squared), fe_mul(y, lambda_cubed)), lambda)
}

pub fn g_times(k: u128) -> Ge {
    let secret_key = SecretKey::from_slice(&u256(k)).expect("k must be non-zero");

    pk_to_ge(&PublicKey::from_secret_key(SECP256K1, &secret_key))
}

/// `x + p` still fits in 256 bits, so x = 1 can be passed unreduced to test reduction mod p.
pub fn x_one() -> Ge {
    decompress((0, ONE)).expect("x = 1 is on the curve")
}
