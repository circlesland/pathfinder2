use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Div;
use std::ops::Mul;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use num_bigint::BigUint;

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct U256([u128; 2]);

impl U256 {
    pub const fn new(high: u128, low: u128) -> U256 {
        U256([high, low])
    }
    pub const MAX: U256 = U256::new(u128::MAX, u128::MAX);
    pub fn from_bigint_truncating(input: BigUint) -> U256 {
        let digits = input.to_u64_digits();
        U256([
            u128::from(*digits.get(3).unwrap_or(&0)) << 64
                | u128::from(*digits.get(2).unwrap_or(&0)),
            u128::from(*digits.get(1).unwrap_or(&0)) << 64
                | u128::from(*digits.first().unwrap_or(&0)),
        ])
    }

    pub fn to_decimal(self) -> String {
        let value = BigUint::from(self.0[0]) << 128 | BigUint::from(self.0[1]);
        format!("{value}")
    }

    pub fn to_decimal_fraction(self) -> String {
        let value: BigUint = self.into();
        let formatted = format!("{value}");
        match formatted.len() {
            18.. => {
                format!(
                    "{}.{}",
                    &formatted[..formatted.len() - 18],
                    &formatted[formatted.len() - 18..formatted.len() - 16]
                )
            }
            17 => {
                format!(".0{}", &formatted[0..1],)
            }
            _ => "0+eps".to_string(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for i in 0..=1 {
            for j in (0..16).rev() {
                let b = ((self.0[i] >> (j * 8)) & 0xff) as u8;
                if b != 0 || !result.is_empty() {
                    result.push(b);
                }
            }
        }
        result
    }
}

impl From<u128> for U256 {
    fn from(item: u128) -> Self {
        U256([0, item])
    }
}

// TODO str is using unicode stuff - maybe we should use Vec<u8> for efficiency reasons?
impl From<&str> for U256 {
    fn from(item: &str) -> Self {
        if item.starts_with("0x") {
            let len = item.len();
            assert!(len <= 2 + 32 + 32, "{}", len);
            let low_start = if len >= 2 + 32 { len - 32 } else { 2 };
            let low_hex = &item[low_start..];
            // disallow + and - prefixes
            assert!(
                low_hex.as_bytes().first() != Some(&54) && low_hex.as_bytes().first() != Some(&43)
            );
            let low = if low_hex.is_empty() {
                0
            } else {
                u128::from_str_radix(low_hex, 16).unwrap()
            };
            let high_start = if len >= 2 + 32 + 32 { len - 64 } else { 2 };
            let high_hex = &item[high_start..low_start];
            // disallow + and - prefixes
            assert!(
                high_hex.as_bytes().first() != Some(&54)
                    && high_hex.as_bytes().first() != Some(&43)
            );
            let high = if high_hex.is_empty() {
                0
            } else {
                u128::from_str_radix(high_hex, 16).unwrap()
            };
            U256([high, low])
        } else {
            let digits = item.parse::<num_bigint::BigUint>().unwrap().to_u64_digits();
            assert!(digits.len() <= 4);
            U256([
                u128::from(*digits.get(3).unwrap_or(&0)) << 64
                    | u128::from(*digits.get(2).unwrap_or(&0)),
                u128::from(*digits.get(1).unwrap_or(&0)) << 64
                    | u128::from(*digits.first().unwrap_or(&0)),
            ])
        }
    }
}

impl From<U256> for BigUint {
    fn from(value: U256) -> Self {
        BigUint::from(value.0[0]) << 128 | BigUint::from(value.0[1])
    }
}

impl Add for U256 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let (low, carry) = self.0[1].overflowing_add(rhs.0[1]);
        let mut high = self.0[0].wrapping_add(rhs.0[0]);
        if carry {
            high = high.wrapping_add(1);
        }
        Self([high, low])
    }
}

impl AddAssign for U256 {
    fn add_assign(&mut self, rhs: U256) {
        *self = *self + rhs;
    }
}

impl Neg for U256 {
    type Output = Self;
    fn neg(self) -> Self {
        let result = U256([!self.0[0], !self.0[1]]);
        result + U256::from(1)
    }
}

impl Sub for U256 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl Mul for U256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let self_big: BigUint = self.into();
        let rhs_big: BigUint = rhs.into();
        U256::from_bigint_truncating(self_big * rhs_big)
    }
}

impl Div for U256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let self_big: BigUint = self.into();
        let rhs_big: BigUint = rhs.into();
        U256::from_bigint_truncating(self_big / rhs_big)
    }
}

impl SubAssign for U256 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Display for U256 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.0[0] == 0 {
            write!(f, "{:#x}", self.0[1])
        } else {
            write!(f, "{:#x}{:032x}", self.0[0], self.0[1])
        }
    }
}

impl Debug for U256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod test {
    use super::U256;
    #[test]
    fn to_string() {
        assert_eq!(format!("{}", U256::from(0)), "0x0");
        assert_eq!(
            format!("{}", U256::from(u128::MAX)),
            "0xffffffffffffffffffffffffffffffff"
        );
    }

    #[test]
    fn add() {
        assert_eq!(
            format!("{}", U256::from(u128::MAX) + U256::from(u128::MAX)),
            "0x1fffffffffffffffffffffffffffffffe"
        );
        let mut x = U256::from(u128::MAX);
        x += U256::from(1);
        assert_eq!(format!("{x}"), "0x100000000000000000000000000000000");
    }

    #[test]
    fn compare() {
        assert!(U256::from(0) < U256::from(1));
        assert!(U256::from("0x100000000000000000000000000000000") > U256::from(1));
    }

    #[test]
    fn from_hex() {
        assert_eq!(U256::from("0x"), U256::from(0));
        assert_eq!(U256::from("0x1"), U256::from(1));
        assert_eq!(U256::from("0x01"), U256::from(1));
        assert_eq!(
            U256::from("0x1fffffffffffffffffffffffffffffffe"),
            U256::from(u128::MAX) + U256::from(u128::MAX)
        );
        assert_eq!(
            U256::from("0x001fffffffffffffffffffffffffffffffe"),
            U256::from(u128::MAX) + U256::from(u128::MAX)
        );
        assert_eq!(
            U256::from("0x100000000000000000000000000000000"),
            U256::from(u128::MAX) + U256::from(1)
        );
    }

    #[test]
    fn from_decimal() {
        assert_eq!(U256::from("0"), U256::from(0));
        assert_eq!(U256::from("10"), U256::from(10));
        assert_eq!(
            U256::from("680564733841876926926749214863536422910"),
            U256::from(u128::MAX) + U256::from(u128::MAX)
        );
        assert_eq!(
            U256::from("000680564733841876926926749214863536422910"),
            U256::from(u128::MAX) + U256::from(u128::MAX)
        );
        assert_eq!(
            U256::from("340282366920938463463374607431768211456"),
            U256::from(u128::MAX) + U256::from(1)
        );
    }

    #[test]
    fn to_decimal() {
        assert_eq!(U256::from("0").to_decimal(), "0");
        assert_eq!(
            U256::from("680564733841876926926749214863536422910").to_decimal(),
            "680564733841876926926749214863536422910"
        );
        assert_eq!(
            U256::from("000680564733841876926926749214863536422910").to_decimal(),
            "680564733841876926926749214863536422910"
        );
        assert_eq!(
            U256::from("340282366920938463463374607431768211456").to_decimal(),
            "340282366920938463463374607431768211456"
        );
    }

    #[test]
    fn to_mul_div() {
        let two = U256::from("2");
        let three = U256::from(3);
        let large = U256::from("0x100000000000000000000000000000000");
        assert_eq!(two * three, U256::from(6));
        assert_eq!(three / two, U256::from(1));
        assert_eq!((large * two) / two, large);
        assert_eq!(
            large / three,
            U256::from("0x55555555555555555555555555555555")
        );
        assert_eq!(large * large, U256::from("0"));
        assert_eq!(
            (large / two) * large,
            U256::from("0x8000000000000000000000000000000000000000000000000000000000000000")
        );
    }

    #[test]
    fn to_bytes() {
        let zero = U256::from("0");
        assert_eq!(zero.to_bytes(), Vec::<u8>::new());
        assert_eq!(U256::from("2").to_bytes(), vec![2]);
        assert_eq!(
            U256::from("0x100000000000000000000000000000000").to_bytes(),
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            U256::from("0xff00000000000000000000000000000001").to_bytes(),
            vec![255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(
            U256::from("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .to_bytes(),
            vec![255; 32]
        );
    }
}
