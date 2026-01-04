use std::{
    cmp::{self, Ordering},
    ops,
};

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Vect<const N: usize>(pub [f32; N]);

impl<const N: usize> Default for Vect<N> {
    fn default() -> Self {
        Self([0.0; N])
    }
}

impl<const N: usize> Vect<N> {
    pub const fn zero() -> Self {
        Self([0.0; N])
    }
    pub const fn one() -> Self {
        Self([1.0; N])
    }
    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum()
    }
    #[inline]
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }
    #[inline]
    pub fn length_sq(&self) -> f32 {
        self.dot(self)
    }
    #[inline]
    pub fn scale(self, k: f32) -> Self {
        self * k
    }
    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            *self * (1.0 / len)
        }
    }
    #[inline]
    pub fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(f32) -> f32,
    {
        Self(std::array::from_fn(|i| f(self.0[i])))
    }

    #[inline]
    pub fn abs(self) -> Self {
        self.map(|x| x.abs())
    }
    #[inline]
    pub fn floor(self) -> Self {
        self.map(|x| x.floor())
    }
    #[inline]
    pub fn ceil(self) -> Self {
        self.map(|x| x.ceil())
    }
    #[inline]
    pub fn round(self, digits_of_precision: Option<u32>) -> Self {
        let factor = 10f32.powi(digits_of_precision.unwrap_or(0) as i32);
        self.map(|x| (x * factor).round() / factor)
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i].max(other.0[i])))
    }
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i].min(other.0[i])))
    }
    #[inline]
    pub fn lerp(self, other: Self, k: f32) -> Self {
        Self(std::array::from_fn(|i| {
            self.0[i] + (other.0[i] - self.0[i]) * k
        }))
    }
    #[inline]
    pub fn sign(self) -> Self {
        Self(std::array::from_fn(|i| self.0[i].signum()))
    }
    // hyper-area in n dimensions where n is 2
    #[inline]
    pub fn harea(self) -> f32 {
        2.0 * std::array::from_fn::<f32, N, _>(|i| {
            std::array::from_fn::<f32, N, _>(|j| if i == j { 1.0 } else { self.0[j] })
                .iter()
                .product()
        })
        .iter()
        .sum::<f32>()
    }
    // hyper-volume in n dimensions where n is 2
    #[inline]
    pub fn hvolume(self) -> f32 {
        self.0.iter().product()
    }
}

impl<const N: usize> ops::Add for Vect<N> {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] + other.0[i]))
    }
}

impl<const N: usize> ops::Sub for Vect<N> {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] - other.0[i]))
    }
}

impl<const N: usize> ops::Mul for Vect<N> {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] * other.0[i]))
    }
}

impl<const N: usize> ops::Mul<f32> for Vect<N> {
    type Output = Self;
    #[inline]
    fn mul(self, k: f32) -> Self {
        Self(std::array::from_fn(|i| self.0[i] * k))
    }
}

impl<const N: usize> ops::Div<f32> for Vect<N> {
    type Output = Self;
    #[inline]
    fn div(self, k: f32) -> Self {
        Self(std::array::from_fn(|i| self.0[i] / k))
    }
}

impl<const N: usize> cmp::PartialOrd for Vect<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut is_less = true;
        let mut is_greater = true;
        for (i, _) in self.0.iter().enumerate() {
            if self.0[i] < other.0[i] {
                is_greater = false
            }
            if self.0[i] > other.0[i] {
                is_less = false
            }
        }
        if is_less {
            return Some(Ordering::Less);
        }
        if is_greater {
            return Some(Ordering::Greater);
        }
        None
    }
}
