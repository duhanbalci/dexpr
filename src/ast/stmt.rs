use smol_str::SmolStr;

use super::expr::{Expr, Spanned};

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
  Assignment(SmolStr, Box<Expr>),
  /// Property assignment: root variable, field path, value
  /// e.g. `a.b.c = 5` → PropertyAssignment("a", ["b", "c"], 5)
  PropertyAssignment(SmolStr, Vec<SmolStr>, Box<Expr>),
  ExprStmt(Box<Expr>),
  If(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>),
}

/// Statement with source location
pub type SpannedStmt = Spanned<Stmt>;