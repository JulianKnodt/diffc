use crate::{BOp, Expr, UOp};
use std::ops::{Add, Div, Mul, Neg, Sub};

impl Mul for Expr {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::BOp(BOp::Mul, box self, box o)
    }
}

impl Add for Expr {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::BOp(BOp::Add, box self, box o)
    }
}

impl Sub for Expr {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::BOp(BOp::Add, box self, box -o)
    }
}

impl Neg for Expr {
    type Output = Self;
    fn neg(self) -> Self {
        Self::UOp(UOp::Neg, box self)
    }
}

impl Div for Expr {
    type Output = Self;
    fn div(self, o: Self) -> Self {
        self * o.recip()
    }
}
