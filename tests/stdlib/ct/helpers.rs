use num_bigint::BigUint;
use rand::{Rng, RngCore, rngs::OsRng};
use secp256k1_zkp::{
    All, Generator, PedersenCommitment, Secp256k1, SecretKey, Tag, Tweak,
    rand::rngs::OsRng as SecpOsRng,
};

/// A confidential asset or amount, as the Elements jets return it.
pub type ConfPoint = (u8, [u8; 32]);
/// Affine coordinates `(x, y)`.
pub type Ge = ([u8; 32], [u8; 32]);

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
const SECP_P: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFC, 0x2F,
];

// FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
const SECP_N: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B, 0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

pub fn secp() -> Secp256k1<All> {
    Secp256k1::new()
}

fn secp_p() -> BigUint {
    BigUint::from_bytes_be(&SECP_P)
}

fn secp_n() -> BigUint {
    BigUint::from_bytes_be(&SECP_N)
}

pub fn to_32_be(x: &BigUint) -> [u8; 32] {
    let b = x.to_bytes_be();

    let mut out = [0u8; 32];
    out[32 - b.len()..].copy_from_slice(&b);

    out
}

pub fn random_scalar() -> BigUint {
    BigUint::from_bytes_be(&SecretKey::new(&mut SecpOsRng).secret_bytes())
}

pub fn random_amount() -> u64 {
    rand::thread_rng().gen_range(1..=1_000_000_000)
}

pub fn random_asset_id() -> [u8; 32] {
    let mut id = [0u8; 32];
    OsRng.fill_bytes(&mut id);

    id
}

/// `a + b mod n`
pub fn add_n(a: &BigUint, b: &BigUint) -> BigUint {
    (a + b) % secp_n()
}

/// `a - b mod n`
pub fn sub_n(a: &BigUint, b: &BigUint) -> BigUint {
    let n = secp_n();
    (a % &n + &n - b % &n) % &n
}

/// `a * b mod n`
pub fn mul_n(a: &BigUint, b: &BigUint) -> BigUint {
    (a * b) % secp_n()
}

pub fn scalar(s: &BigUint) -> [u8; 32] {
    to_32_be(&(s % secp_n()))
}

fn tweak(s: &BigUint) -> Tweak {
    Tweak::from_inner(scalar(s)).expect("non-zero scalar below n")
}

/// The unblinded asset generator `H_0 = hash_to_curve(asset_id)`.
pub fn unblinded_generator(asset_id: [u8; 32]) -> Generator {
    Generator::new_unblinded(&secp(), Tag::from(asset_id))
}

/// The blinded asset generator `H_a = H_0 + abf*G`.
pub fn blinded_generator(asset_id: [u8; 32], abf: &BigUint) -> Generator {
    Generator::new_blinded(&secp(), Tag::from(asset_id), tweak(abf))
}

/// The value commitment `C = v*generator + vbf*G`.
pub fn commitment(v: u64, generator: Generator, vbf: &BigUint) -> PedersenCommitment {
    PedersenCommitment::new(&secp(), v, tweak(vbf), generator)
}

/// The unblinded commitment `v*generator` that Elements uses for an explicit amount.
pub fn unblinded_commitment(v: u64, generator: Generator) -> PedersenCommitment {
    PedersenCommitment::new_unblinded(&secp(), v, generator)
}

/// The blinding factor of `v*(H_0 + abf*G) + vbf*G` relative to `H_0`, i.e. `v*abf + vbf`.
/// Relations between commitments of the same asset hold up to these factors.
pub fn total_blinding(v: u64, abf: &BigUint, vbf: &BigUint) -> BigUint {
    add_n(&mul_n(&BigUint::from(v), abf), vbf)
}

/// The Elements encoding of a serialized generator or commitment:
/// `bit == 1` iff y is not a square.
pub fn conf_point(serialized: [u8; 33]) -> ConfPoint {
    (serialized[0] & 1, serialized[1..].try_into().unwrap())
}

/// Reference decoder for `conf_point`: affine coordinates of a serialized
/// generator or commitment.
pub fn decode(serialized: [u8; 33]) -> Ge {
    let p = secp_p();
    let x = BigUint::from_bytes_be(&serialized[1..]);
    let y_squared = (&x * &x * &x + 7u32) % &p;

    // `a^((p+1)/4)` is the square root that is itself a square.
    let y = y_squared.modpow(&((&p + 1u32) >> 2), &p);
    let y = if serialized[0] & 1 == 1 { &p - y } else { y };

    (to_32_be(&x), to_32_be(&y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1_zkp::{PublicKey, SecretKey};

    // A commitment to 0 is `vbf*G`, i.e. the public key of `vbf`, whose
    // coordinates `secp256k1` gives us independently.
    #[test]
    fn decode_matches_public_key() {
        let generator = unblinded_generator(random_asset_id());

        for _ in 0..1_000 {
            let vbf = random_scalar();
            let c = commitment(0, generator, &vbf).serialize();

            let sk = SecretKey::from_slice(&scalar(&vbf)).unwrap();
            let pk = PublicKey::from_secret_key(&secp(), &sk).serialize_uncompressed();

            assert_eq!(
                decode(c),
                (pk[1..33].try_into().unwrap(), pk[33..].try_into().unwrap())
            );
        }
    }
}
