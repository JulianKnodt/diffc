use super::Expr;
use crate::backend::{Backend, SExpr};

#[test]
fn basic_example() {
    let s = Expr::new_var(0) * (Expr::ONE + Expr::new_var(1));
    let s_diff = s.diff(1);

    let mut out_sexpr = SExpr::new();
    out_sexpr.emit_expr(s).unwrap();

    let mut out_sexpr2 = SExpr::new();
    out_sexpr2.emit_expr(s_diff).unwrap();
    panic!("{} -> {}", out_sexpr.finish(), out_sexpr2.finish());
}
