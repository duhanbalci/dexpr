use crate::ast::expr::Span;
use smol_str::SmolStr;
use thiserror::Error;

/// Errors that can occur during VM execution
#[derive(Debug, Error)]
pub enum VMError {
  /// Error when types don't match the operation's expectations
  #[error("Type error: expected {expected}, got {got}")]
  TypeMismatch { expected: String, got: String },

  /// Error when a variable is not defined
  #[error("Undefined variable: {0}")]
  UndefinedVariable(SmolStr),

  /// Error when dividing by zero
  #[error("Division by zero")]
  DivisionByZero,

  /// Error in bytecode format or execution
  #[error("Bytecode error: {0}")]
  BytecodeError(String),

  /// Error when a method is not found for a type
  #[error("Method '{method}' not found for type '{type_name}'")]
  MethodNotFound {
    type_name: &'static str,
    method: SmolStr,
  },

  /// Generic runtime error
  #[error("Runtime error: {0}")]
  RuntimeError(String),

  /// Error when an invalid operation is performed
  #[error("Type error: cannot {operation} {left_type} and {right_type}")]
  InvalidOperation {
    operation: &'static str,
    left_type: &'static str,
    right_type: &'static str,
  },

  /// Error with source location information
  #[error("Error at {span}: {message}")]
  WithLocation { span: Span, message: String },
}

impl VMError {
  /// Wrap this error with source location information
  pub fn with_span(self, span: Span) -> Self {
    // Don't double-wrap location errors
    if matches!(self, VMError::WithLocation { .. }) {
      return self;
    }
    // Only wrap if we have a valid span (non-zero)
    if span.line == 0 && span.column == 0 {
      return self;
    }
    VMError::WithLocation {
      span,
      message: self.to_string(),
    }
  }
}
