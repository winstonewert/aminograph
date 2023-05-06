use crate::prelude::*;
use statrs::function::beta::ln_beta;
use statrs::function::gamma::ln_gamma;

#[derive(Copy, Clone, Ord, Eq, PartialOrd, PartialEq, Serialize, Deserialize, Default)]
pub struct Log(N64);

impl std::convert::From<f64> for Log {
    fn from(value: f64) -> Self {
        Log(n64(value.log2()))
    }
}

impl std::convert::From<N64> for Log {
    fn from(value: N64) -> Self {
        Log(value.log2())
    }
}

impl std::convert::From<R64> for Log {
    fn from(value: R64) -> Self {
        Log(n64(value.raw().log2()))
    }
}

impl std::convert::From<i32> for Log {
    fn from(value: i32) -> Self {
        Log(n64((value as f64).log2()))
    }
}

impl std::convert::From<u32> for Log {
    fn from(value: u32) -> Self {
        Log(n64((value as f64).log2()))
    }
}

impl std::convert::From<usize> for Log {
    fn from(value: usize) -> Self {
        Log(n64((value as f64).log2()))
    }
}

impl std::fmt::Debug for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:.2}b", -self.log2()))
    }
}

#[allow(dead_code)]
impl Log {
    pub fn one() -> Log {
        Log(n64(0.0))
    }

    pub fn pow2(value: N64) -> Log {
        Log(value)
    }

    pub fn exp(value: N64) -> Log {
        Log(value / (2.0.ln()))
    }

    pub fn zero() -> Log {
        Log(n64(f64::NEG_INFINITY))
    }

    pub fn powi(self, value: i32) -> Log {
        Log(self.0 * n64(value as f64))
    }

    pub fn nchoosek(n: i32, k: i32) -> Log {
        Log::gammai(n + 1) / (Log::gammai(k + 1) * Log::gammai(n - k + 1))
    }

    pub fn gammai(value: i32) -> Log {
        Log(n64(ln_gamma(value as f64) as f64) / n64(2.0).ln())
    }

    pub fn betai(a: i32, b: i32) -> Log {
        Log(n64(ln_beta(a as f64, b as f64) as f64) / n64(2.0).ln())
    }

    pub fn beta(a: R64, b: R64) -> Log {
        Log(n64(ln_beta(a.raw(), b.raw())) / n64(2.0).ln())
    }

    pub fn log2(self) -> N64 {
        self.0
    }

    pub fn raw(self) -> N64 {
        self.0.exp2()
    }
}

impl std::iter::Sum for Log {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let items = iter.collect_vec();
        let biggest = items.iter().copied().max().unwrap();
        Log::from(
            items
                .into_iter()
                .map(|item| (item / biggest).raw())
                .sum::<N64>(),
        ) * biggest
    }
}

impl std::iter::Product for Log {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        Log(iter.into_iter().map(|x| x.0).sum())
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl std::ops::Mul for Log {
    type Output = Log;
    fn mul(self, rhs: Self) -> Self::Output {
        Log(self.0 + rhs.0)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl std::ops::Div for Log {
    type Output = Log;
    fn div(self, rhs: Self) -> Self::Output {
        Log(self.0 - rhs.0)
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::DivAssign for Log {
    fn div_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::MulAssign for Log {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::AddAssign for Log {
    fn add_assign(&mut self, rhs: Self) {
        let bigger = if *self < rhs { rhs } else { *self };
        *self = Log::from((*self / bigger).raw() + (rhs / bigger).raw()) * bigger
    }
}

impl std::ops::SubAssign for Log {
    fn sub_assign(&mut self, rhs: Self) {
        let bigger = if *self < rhs { rhs } else { *self };
        *self = Log::from((*self / bigger).raw() - (rhs / bigger).raw()) * bigger
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct FixedLog(fixed::FixedI64<fixed::types::extra::U32>);

impl serde::Serialize for FixedLog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.unfix().serialize(serializer)
    }
}

impl FixedLog {
    pub fn one() -> FixedLog {
        FixedLog(fixed::FixedI64::from(0))
    }

    pub fn smallest() -> FixedLog {
        FixedLog(fixed::FixedI64::MIN)
    }

    pub fn unfix(self) -> Log {
        Log(n64(self.0.to_num()))
    }

    #[allow(unused)]
    pub fn gammai(value: i32) -> FixedLog {
        FixedLog(fixed::FixedI64::from_num(
            (ln_gamma(value as f64) as f64) / (2.0).ln(),
        ))
    }
}

impl std::convert::From<f64> for FixedLog {
    fn from(value: f64) -> Self {
        FixedLog(fixed::FixedI64::from_num(value.log2()))
    }
}

impl std::convert::From<R64> for FixedLog {
    fn from(value: R64) -> Self {
        FixedLog(fixed::FixedI64::from_num(value.log2().raw()))
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::MulAssign for FixedLog {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl std::ops::DivAssign for FixedLog {
    fn div_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
