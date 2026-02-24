/// Simple SourceMap representation of a single file
pub struct SourceFile {
  pub(crate) content: String,
  line_starts: Vec<usize>,
}

impl SourceFile {
  /// Creates a new `SourceFile` and builds the line index.
  pub fn new(content: String) -> Self {
    let mut line_starts = vec![0];
    for (i, b) in content.as_bytes().iter().enumerate() {
      if *b == b'\n' {
        line_starts.push(i + 1);
      }
    }
    Self {
      content,
      line_starts,
    }
  }

  /// Returns the content of the source file.  
  pub fn src(&self) -> &str {
    &self.content
  }

  /// # Arguments
  ///
  /// * `byte_offset` - The absolute byte index within the source file's content.
  ///
  /// # Returns
  ///
  /// Returns a tuple `(line, column)` representing the position that is used in IDEs.
  /// Both `line` and `column` are **1-based** (start from 1).
  pub fn lookup_pos(&self, byte_offset: usize) -> (usize, usize) {
    let line_idx = match self.line_starts.binary_search(&byte_offset) {
      Ok(idx) => idx,
      Err(idx) => idx - 1,
    };

    let line_start = self.line_starts[line_idx];
    let line_number = line_idx + 1;

    let mut byte_offset = byte_offset;
    while !self.content.is_char_boundary(byte_offset) {
      byte_offset -= 1;
    }

    // take the prefix and count its utf-16 char num to correctly locate in IDEs
    let prefix = &self.content[line_start..byte_offset];
    let column_number = prefix.chars().map(|c| c.len_utf16()).sum::<usize>() + 1;

    (line_number, column_number)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_source_file_lookup() {
    let code = "int main() {\n    return 0;\n}".to_string();
    let src = SourceFile::new(code);

    assert_eq!(src.lookup_pos(0), (1, 1));
    assert_eq!(src.lookup_pos(4), (1, 5));
    assert_eq!(src.lookup_pos(12), (1, 13));
    assert_eq!(src.lookup_pos(13), (2, 1));
    assert_eq!(src.lookup_pos(17), (2, 5));
    assert_eq!(src.lookup_pos(27), (3, 1));
  }

  #[test]
  fn test_empty_file() {
    let src = SourceFile::new("".to_string());
    assert_eq!(src.lookup_pos(0), (1, 1));
  }

  #[test]
  fn test_utf() {
    let code = "int main() {\n  float /* 好耶😆 */ a;\n}";
    let src = SourceFile::new(code.to_string());

    let pos_of = |s: &str| code.find(s).unwrap();

    // start of first line
    assert_eq!(src.lookup_pos(0), (1, 1));
    // start of second line
    assert_eq!(src.lookup_pos(pos_of("  float")), (2, 1));
    assert_eq!(src.lookup_pos(pos_of("float")), (2, 3));
    assert_eq!(src.lookup_pos(pos_of("好")), (2, 12));
    assert_eq!(src.lookup_pos(pos_of("耶")), (2, 13));
    assert_eq!(src.lookup_pos(pos_of("😆")), (2, 14));
    assert_eq!(src.lookup_pos(pos_of("a;")), (2, 20));
  }
}
