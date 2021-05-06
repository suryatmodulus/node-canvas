use regex::Regex;

#[inline(always)]
pub(crate) fn init_filter_regexp() -> Regex {
  Regex::new(
    r#"(?x)
    ((blur\(([\d\.]+)(%|px|pt|pc|in|cm|mm|%|em|ex|ch|rem|q)?\)(\s+|$)){0,1})|
    (((brightness|contrast|grayscale|invert|opacity|saturate|sepia)\(\s*([\d\.]+)\s*\)(\s+|$)){0,1})|
    ((hue\-rotate\(\s*([\d\.]+)(deg|turn|rad)?\)(\s+|$)){0,1}) |
    ((drop\-shadow\((([\d\.]+)(%|px|pt|pc|in|cm|mm|%|em|ex|ch|rem|q)?)\s+(([\d\.]+)(%|px|pt|pc|in|cm|mm|%|em|ex|ch|rem|q)?)\s+(([\d\.]+)(%|px|pt|pc|in|cm|mm|%|em|ex|ch|rem|q)?)?\)){0,1})
  "#,
  )
  .unwrap()
}

mod test {
  #[test]
  fn init_filter_regexp() {
    super::init_filter_regexp();
  }
}
