use crate::ast::expr::Span;

/// Debug information that maps bytecode offsets to source locations.
/// Uses a run-length encoded format: each entry covers instructions from
/// its offset until the next entry's offset.
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
  /// Sorted list of (bytecode_offset, span) pairs
  entries: Vec<(u32, Span)>,
}

impl DebugInfo {
  pub fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Add a mapping from a bytecode offset to a source span.
  /// Entries must be added in increasing offset order.
  pub fn add_entry(&mut self, offset: u32, span: Span) {
    // Only add if different from the last entry's span
    if let Some((_, last_span)) = self.entries.last() {
      if *last_span == span {
        return;
      }
    }
    self.entries.push((offset, span));
  }

  /// Look up the source span for a given bytecode offset.
  /// Returns None if no debug info is available.
  pub fn get_span(&self, offset: u32) -> Option<Span> {
    if self.entries.is_empty() {
      return None;
    }

    // Binary search for the largest offset <= target
    match self.entries.binary_search_by_key(&offset, |(off, _)| *off) {
      Ok(idx) => Some(self.entries[idx].1),
      Err(idx) => {
        if idx == 0 {
          // Before the first entry
          None
        } else {
          // Use the previous entry
          Some(self.entries[idx - 1].1)
        }
      }
    }
  }

  /// Check if debug info is available
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Get the number of entries
  pub fn len(&self) -> usize {
    self.entries.len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_debug_info_lookup() {
    let mut info = DebugInfo::new();
    info.add_entry(0, Span::new(1, 1));
    info.add_entry(10, Span::new(2, 5));
    info.add_entry(20, Span::new(3, 10));

    // Exact matches
    assert_eq!(info.get_span(0), Some(Span::new(1, 1)));
    assert_eq!(info.get_span(10), Some(Span::new(2, 5)));
    assert_eq!(info.get_span(20), Some(Span::new(3, 10)));

    // In-between values use previous entry
    assert_eq!(info.get_span(5), Some(Span::new(1, 1)));
    assert_eq!(info.get_span(15), Some(Span::new(2, 5)));
    assert_eq!(info.get_span(100), Some(Span::new(3, 10)));
  }

  #[test]
  fn test_duplicate_spans_not_added() {
    let mut info = DebugInfo::new();
    info.add_entry(0, Span::new(1, 1));
    info.add_entry(5, Span::new(1, 1)); // Same span, should not add
    info.add_entry(10, Span::new(2, 1));

    assert_eq!(info.len(), 2);
  }
}
