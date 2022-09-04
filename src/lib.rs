#![feature(box_syntax, box_patterns)]
#[macro_use]
extern crate lalrpop_util;

#[cfg(test)]
mod tests;

/// Variable ID
pub type ID = u32;

// A simple frontend for ingesting expressions.
lalrpop_mod!(pub frontend);

/// How to emit an expression to some backend.
/// For example, emitting valid rust code or s-expressions.
pub mod backend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BOp {
    Add,
    Mul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UOp {
    Neg,
    Abs,
    Recip,
    // TODO maybe make heavside part of this?
}

#[macro_export]
macro_rules! H {
    ($v: expr) => {
        Expr::Heavside(box $v)
    };
    (inv $v: expr) => {
        H!(Expr::ONE + -$v)
    };
}

// TODO add an actual guard for this
macro_rules! guard {
    ($v: expr) => {
        Expr::Guard(box $v)
    };
}

// TODO maybe make this generic over different values
/// A generic expression
#[derive(Debug, Clone)]
pub enum Expr {
    Const(f64),
    Var {
        id: ID,
        sampling_axis: bool,
    },
    BOp(BOp, Box<Expr>, Box<Expr>),
    UOp(UOp, Box<Expr>),

    Heavside(Box<Expr>),
    // TODO add a "Guard" into this which is not differentiable but adds a small epsilon to
    // denominators.
    /// Guard, adding a small epsilon to denominators generated during autodiff
    Guard(Box<Expr>),
}

pub mod ops;

const EPS: f64 = 1e-4;

impl Expr {
    pub const ZERO: Self = Expr::Const(0.);
    pub const ONE: Self = Expr::Const(1.);
    pub const HALF: Self = Expr::Const(0.5);

    /// Creates a new variable, with a unique ID. For testing use only.
    pub fn new_var(id: u32) -> Self {
        Self::Var {
            id,
            sampling_axis: false,
        }
    }

    pub fn new_sampling_axis(id: u32) -> Self {
        Self::Var {
            id,
            sampling_axis: true,
        }
    }

    /// Evaluates this expression where possible.
    pub fn eval(&self, resolve: impl Fn(u32) -> f64) -> f64 {
        use Expr::*;
        match self {
            &Const(v) => v,
            &Var { id, .. } => resolve(id),
            BOp(bop, lhs, rhs) => {
                let lhs = lhs.eval(&resolve);
                let rhs = rhs.eval(resolve);
                match bop {
                    crate::BOp::Add => lhs + rhs,
                    crate::BOp::Mul => lhs * rhs,
                }
            }
            UOp(uop, v) => {
                let v = v.eval(resolve);
                match uop {
                    crate::UOp::Abs => v.abs(),
                    crate::UOp::Recip => v.recip(),
                    crate::UOp::Neg => -v,
                }
            }
            Guard(v) => {
                let v = v.eval(resolve);
                v + EPS.copysign(v)
            }
            Heavside(v) => heavside(v.eval(resolve)),
        }
    }

    /// Recursively applies a small shift to a specific parameter id during evaluation.
    pub fn shift(&self, id: u32, amt: f64) -> Self {
        use Expr::*;
        match self {
            &Const(c) => Const(c),
            &Var {
                id: i2,
                sampling_axis,
            } => {
                if sampling_axis {
                    Var {
                        id: i2,
                        sampling_axis,
                    } + Const(amt)
                } else {
                    self.clone()
                }
            }
            BOp(bop, lhs, rhs) => BOp(*bop, box lhs.shift(id, amt), box rhs.shift(id, amt)),
            UOp(uop, v) => UOp(*uop, box v.shift(id, amt)),

            Heavside(v) => Heavside(box v.shift(id, amt)),
            Guard(v) => Guard(box v.shift(id, amt)),
        }
    }

    /// Takes the absolute value of this expression
    pub fn abs(self) -> Self {
        Expr::UOp(UOp::Abs, box self)
    }

    /// Takes the reciprocal of this expression
    pub fn recip(self) -> Self {
        Expr::UOp(UOp::Recip, box self)
    }

    /// Takes `1 - v` of this Expression
    pub fn fuzzy_not(self) -> Self {
        Expr::ONE - self
    }

    /// Heavside function of this expression
    pub fn heavside(self) -> Self {
        H!(self)
    }

    /// Converts this expression into an abstract representation of a differentiated expression,
    /// w.r.t. a given ID
    pub fn diff(&self, id: ID) -> Expr {
        use Expr::*;
        match self {
            Const(..) => Expr::ZERO,
            Var { id: i2, .. } => {
                if *i2 == id {
                    Expr::ONE
                } else {
                    Expr::ZERO
                }
            }
            BOp(bop, lhs, rhs) => {
                use crate::BOp::*;
                match bop {
                    Add => BOp(Add, box lhs.diff(id), box rhs.diff(id)),
                    Mul => {
                        let lhs_l_lim = lhs.shift(id, -EPS);
                        let lhs_r_lim = lhs.shift(id, EPS);

                        let rhs_l_lim = rhs.shift(id, -EPS);
                        let rhs_r_lim = rhs.shift(id, EPS);

                        Expr::HALF * (rhs_l_lim + rhs_r_lim) * lhs.diff(id)
                            + Expr::HALF * (lhs_l_lim + lhs_r_lim) * rhs.diff(id)
                    }
                }
            }
            UOp(uop, box v) => {
                use crate::UOp::*;
                let k = match uop {
                    Neg => -Expr::ONE,
                    Recip => -(v.clone() * v.clone()).recip(),
                    Abs => {
                        let r_lim = v.shift(id, EPS);
                        let l_lim = v.shift(id, -EPS);
                        (r_lim.clone().abs() - l_lim.clone().abs()) / guard!(r_lim - l_lim)
                    }
                };
                k * v.diff(id)
            }
            Heavside(e) => {
                let left_lim = e.shift(id, -EPS);
                let right_lim = e.shift(id, EPS);
                H!(inv left_lim.clone() * right_lim.clone())
                    * e.diff(id)
                    * (left_lim - right_lim).abs().recip()
            }
            // having extra guards doesn't hurt? Also probably still need it in most cases.
            Guard(v) => guard!(v.diff(id)),
        }
    }
}

#[inline]
fn heavside(v: f64) -> f64 {
    if v > 0.0 {
        1.0
    } else {
        0.0
    }
}
