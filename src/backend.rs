use super::{BOp, Expr, UOp};
use std::fmt::Write;

/// A method for emitting expressions
pub trait Backend {
    type Output;
    type Err;
    fn emit_expr(&mut self, v: Expr) -> Result<(), Self::Err>;

    /// Complete emitting the Expr,
    fn finish(self) -> Self::Output;
}

#[derive(Default, Debug, Clone)]
pub struct SExpr {
    buffer: String,
}

impl SExpr {
    pub fn new() -> Self {
        SExpr {
            buffer: String::new(),
        }
    }
}

impl Backend for SExpr {
    type Output = String;
    type Err = std::fmt::Error;
    fn emit_expr(&mut self, v: Expr) -> Result<(), Self::Err> {
        match v {
            Expr::Const(v) => write!(&mut self.buffer, " {v}")?,
            Expr::Var { id, .. } => write!(&mut self.buffer, " x{id}")?,
            Expr::BOp(bop, box lhs, box rhs) => {
                write!(&mut self.buffer, " (")?;
                match bop {
                    BOp::Add => write!(&mut self.buffer, "+")?,
                    BOp::Mul => write!(&mut self.buffer, "*")?,
                }
                self.emit_expr(lhs)?;
                self.emit_expr(rhs)?;
                write!(&mut self.buffer, ")")?;
            }
            Expr::UOp(uop, box v) => {
                write!(&mut self.buffer, " (")?;
                match uop {
                    UOp::Neg => write!(&mut self.buffer, " -")?,
                    UOp::Abs => write!(&mut self.buffer, "abs")?,
                    UOp::Recip => write!(&mut self.buffer, "recip")?,
                }
                self.emit_expr(v)?;
                write!(&mut self.buffer, ")")?;
            }
            // It's ok to just ignore guards?
            Expr::Guard(box v) => self.emit_expr(v)?,
            Expr::Heavside(box v) => {
                write!(&mut self.buffer, " (")?;
                write!(&mut self.buffer, "heavside")?;
                self.emit_expr(v)?;
                write!(&mut self.buffer, ")")?;
            }
        }
        Ok(())
    }

    /// Complete emitting the Expr,
    fn finish(self) -> Self::Output {
        self.buffer
    }
}
