use primitive_types::U256;
use rand::Rng;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use std::ops::{Add, Deref, DerefMut, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct U256Wrapper(pub U256);

impl U256Wrapper {
    pub fn to_be_bytes(&self) -> [u8; 32] {
        self.0.to_big_endian()
    }
}

impl Deref for U256Wrapper {
    type Target = U256;
    fn deref(&self) -> &U256 {
        &self.0
    }
}

impl DerefMut for U256Wrapper {
    fn deref_mut(&mut self) -> &mut U256 {
        &mut self.0
    }
}

impl From<U256> for U256Wrapper {
    fn from(v: U256) -> Self {
        U256Wrapper(v)
    }
}

impl From<U256Wrapper> for U256 {
    fn from(v: U256Wrapper) -> Self {
        v.0
    }
}

impl Add for U256Wrapper {
    type Output = U256Wrapper;
    fn add(self, rhs: Self) -> Self::Output {
        U256Wrapper(self.0 + rhs.0)
    }
}

impl Sub for U256Wrapper {
    type Output = U256Wrapper;
    fn sub(self, rhs: Self) -> Self::Output {
        U256Wrapper(self.0 - rhs.0)
    }
}

impl Mul for U256Wrapper {
    type Output = U256Wrapper;
    fn mul(self, rhs: Self) -> Self::Output {
        U256Wrapper(self.0 * rhs.0)
    }
}

impl Div for U256Wrapper {
    type Output = U256Wrapper;
    fn div(self, rhs: Self) -> Self::Output {
        U256Wrapper(self.0 / rhs.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UniformU256 {
    lower: U256,
    range: U256,
    inclusive_max: bool,
}

impl UniformSampler for UniformU256 {
    type X = U256Wrapper;

    fn new<B1, B2>(lower: B1, upper: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let lower = lower.borrow().0;
        let upper = upper.borrow().0;

        assert!(lower < upper, "Lower bound must be less than upper bound");

        UniformU256 {
            lower,
            range: upper - lower,
            inclusive_max: false,
        }
    }

    fn new_inclusive<B1, B2>(lower: B1, upper: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let lower = lower.borrow().0;
        let upper = upper.borrow().0;

        assert!(
            lower <= upper,
            "Lower bound must be less than or equal to upper bound"
        );

        if upper == U256::MAX && lower == U256::zero() {
            UniformU256 {
                lower,
                range: U256::zero(),
                inclusive_max: true,
            }
        } else {
            UniformU256 {
                lower,
                range: upper - lower + U256::one(),
                inclusive_max: false,
            }
        }
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        if self.inclusive_max {
            return U256Wrapper(random_u256(rng));
        }
        loop {
            let candidate = random_u256(rng);
            let result = candidate % self.range;
            let limit = U256::MAX - (U256::MAX % self.range);

            if candidate < limit {
                return U256Wrapper(self.lower + result);
            }
        }
    }
}

impl SampleUniform for U256Wrapper {
    type Sampler = UniformU256;
}

fn random_u256<R: Rng + ?Sized>(rng: &mut R) -> U256 {
    let mut bytes = [0u8; 32];

    rng.fill(&mut bytes);

    U256::from_big_endian(&bytes)
}
