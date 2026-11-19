//! Checked rational arithmetic shared by exact matrices and interpolation.
use super::{Signed, Unsigned, Work, invalid, overflow};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FractionInput {
    pub numerator: Signed,
    pub denominator: Unsigned,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Scalar {
    Integer(i64),
    WideInteger(Signed),
    Fraction(FractionInput),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
pub struct Rational {
    pub numerator: Signed,
    pub denominator: Unsigned,
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Scalar {
    pub(super) fn reduced(self) -> Result<Rational, OperationError> {
        match self {
            Self::Integer(n) => Ok(Rational::integer(i128::from(n))),
            Self::WideInteger(n) => Ok(Rational::integer(n.0)),
            Self::Fraction(value) => Rational::new(
                value.numerator.0.is_negative(),
                value.numerator.0.unsigned_abs(),
                value.denominator.0,
            ),
        }
    }
}
impl Rational {
    pub(super) const ZERO: Self = Self::integer(0);
    pub(super) const ONE: Self = Self::integer(1);
    pub(super) const fn integer(n: i128) -> Self {
        Self {
            numerator: Signed(n),
            denominator: Unsigned(1),
        }
    }
    fn new(negative: bool, absolute: u128, denominator: u128) -> Result<Self, OperationError> {
        if denominator == 0 {
            return Err(invalid("rational denominator must be positive"));
        }
        let common = gcd(absolute, denominator);
        let absolute = absolute / common;
        let numerator = if negative && absolute == 1 << 127 {
            i128::MIN
        } else {
            let magnitude = i128::try_from(absolute).map_err(|_| overflow())?;
            if negative { -magnitude } else { magnitude }
        };
        Ok(Self {
            numerator: Signed(numerator),
            denominator: Unsigned(denominator / common),
        })
    }
    pub(super) const fn zero(self) -> bool {
        self.numerator.0 == 0
    }
    pub(super) fn add(self, rhs: Self, work: &mut Work) -> Result<Self, OperationError> {
        self.sum(rhs, false, work)
    }
    pub(super) fn sub(self, rhs: Self, work: &mut Work) -> Result<Self, OperationError> {
        self.sum(rhs, true, work)
    }

    fn sum(self, rhs: Self, subtract: bool, work: &mut Work) -> Result<Self, OperationError> {
        work.charge(1)?;
        let (left, right) = (self.denominator.0, rhs.denominator.0);
        let shared = gcd(left, right);
        let a = self
            .numerator
            .0
            .unsigned_abs()
            .checked_mul(right / shared)
            .ok_or_else(overflow)?;
        let b = rhs
            .numerator
            .0
            .unsigned_abs()
            .checked_mul(left / shared)
            .ok_or_else(overflow)?;
        let (p, q) = (
            self.numerator.0.is_negative(),
            rhs.numerator.0.is_negative() ^ subtract,
        );
        let (sign, magnitude) = if p == q {
            (p, a.checked_add(b).ok_or_else(overflow)?)
        } else if a >= b {
            (p, a - b)
        } else {
            (q, b - a)
        };
        // Cancel the remaining shared factor before constructing the denominator.
        let cancel = gcd(magnitude, shared);
        let denominator = (left / shared)
            .checked_mul(right / cancel)
            .ok_or_else(overflow)?;
        Self::new(sign, magnitude / cancel, denominator)
    }

    pub(super) fn mul(self, rhs: Self, work: &mut Work) -> Result<Self, OperationError> {
        work.charge(1)?;
        let sign = self.numerator.0.is_negative() ^ rhs.numerator.0.is_negative();
        multiply(
            sign,
            self.numerator.0.unsigned_abs(),
            self.denominator.0,
            rhs.numerator.0.unsigned_abs(),
            rhs.denominator.0,
        )
    }

    pub(super) fn div(self, rhs: Self, work: &mut Work) -> Result<Self, OperationError> {
        work.charge(1)?;
        if rhs.zero() {
            return Err(invalid("division by zero rational"));
        }
        let sign = self.numerator.0.is_negative() ^ rhs.numerator.0.is_negative();
        multiply(
            sign,
            self.numerator.0.unsigned_abs(),
            self.denominator.0,
            rhs.denominator.0,
            rhs.numerator.0.unsigned_abs(),
        )
    }
}

fn multiply(sign: bool, a: u128, b: u128, c: u128, d: u128) -> Result<Rational, OperationError> {
    let x = gcd(a, d);
    let y = gcd(c, b);
    let numerator = (a / x).checked_mul(c / y).ok_or_else(overflow)?;
    let denominator = (b / y).checked_mul(d / x).ok_or_else(overflow)?;
    Rational::new(sign, numerator, denominator)
}
