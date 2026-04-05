use super::value::Value;
use smol_str::SmolStr;

/// Source code location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
  pub line: u32,
  pub column: u32,
}

impl Span {
  pub fn new(line: u32, column: u32) -> Self {
    Self { line, column }
  }
}

impl std::fmt::Display for Span {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "line {}, column {}", self.line, self.column)
  }
}

/// Wrapper that associates a value with its source location
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
  pub node: T,
  pub span: Span,
}

impl<T> Spanned<T> {
  pub fn new(node: T, span: Span) -> Self {
    Self { node, span }
  }

  pub fn dummy(node: T) -> Self {
    Self {
      node,
      span: Span::default(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Value(Value),
  Variable(SmolStr),
  BinaryOp(Box<Expr>, Op, Box<Expr>),
  UnaryOp(Op, Box<Expr>),
  FunctionCall(SmolStr, Vec<Expr>),
  MethodCall(Box<Expr>, SmolStr, Vec<Expr>),
  PropertyAccess(Box<Expr>, SmolStr),
}

/// Expression with source location
pub type SpannedExpr = Spanned<Expr>;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
  Add,
  Sub,
  Mul,
  Div,
  Mod, // Modulo
  Pow, // Power
  Lt,
  Lte,
  Gt,
  Gte,
  Eq,
  Neq,
  Neg,
  And,
  Or,
  Not,
  In, // Membership test (value in list/string)

}
