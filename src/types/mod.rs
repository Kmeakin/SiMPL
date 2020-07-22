use self::ty::Type;
use crate::hir::Expr;
use std::str::FromStr;

mod constraint;
mod subst;
pub mod ty;
mod unify;

mod pp;

#[cfg(test)]
mod test;

/// Infer the type of the expr
pub fn type_of(expr: &Expr) -> Type {
    let cons = constraint::collect(expr.clone());
    let subst = unify::unify(&cons);
    subst.apply_ty(&expr.ty())
}

/// Infer the type of the expr, and apply the resulting substitution to the
/// expression (so every expr has its inferred type attatched)
pub fn infer_and_apply(expr: &Expr) -> Expr {
    let cons = constraint::collect(expr.clone());
    let subst = unify::unify(&cons);
    expr.apply(&subst)
}

/// Convenience function. Parse source code, and give every expr its inferred
/// type
pub fn parse_and_type(src: &str) -> Expr {
    // TODO: return a &dyn impl Error instead of unwrapping
    let expr = Expr::from_str(src).unwrap();
    infer_and_apply(&expr)
}
