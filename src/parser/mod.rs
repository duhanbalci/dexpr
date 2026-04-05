mod grammar;

use crate::ast::expr::Span;

pub use grammar::parser::program;
pub use grammar::parser::program_with_spans;

/// Convert a byte offset in source code to line and column numbers
/// Lines and columns are 1-indexed
pub fn offset_to_span(source: &str, offset: usize) -> Span {
  let mut line = 1u32;
  let mut col = 1u32;
  let mut current_offset = 0;

  for ch in source.chars() {
    if current_offset >= offset {
      break;
    }
    if ch == '\n' {
      line += 1;
      col = 1;
    } else {
      col += 1;
    }
    current_offset += ch.len_utf8();
  }

  Span::new(line, col)
}