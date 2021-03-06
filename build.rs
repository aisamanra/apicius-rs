use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const FILE_PREFIX: &str = "
#[cfg(test)]
use pretty_assertions::{assert_eq};

use crate::types::*;
use crate::grammar;
use crate::checks;
use crate::render;

// to let us use pretty_assertions with strings, we write a newtype
// that delegates Debug to Display
#[derive(PartialEq, Eq)]
struct StringWrapper<'a> {
  wrapped: &'a str,
}

impl<'a> core::fmt::Debug for StringWrapper<'a> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    core::fmt::Display::fmt(self.wrapped, f)
  }
}

fn assert_eq(x: &str, y: &str) {
  assert_eq!(StringWrapper {wrapped: x}, StringWrapper {wrapped: y});
}
";

const TEST_TEMPLATE: &str = "
// test for %FILE%
#[test]
fn test_%PREFIX%() {
  let source = include_str!(\"%ROOT%/tests/%PREFIX%.apicius\");
  let mut s = State::new();
  let recipe = grammar::RecipeParser::new().parse(&mut s, source);
  assert!(recipe.is_ok());
  let recipe = recipe.unwrap();

  let mut buf = Vec::new();
  s.debug_recipe(&mut buf, &recipe).unwrap();
  assert_eq(
    std::str::from_utf8(&buf).unwrap().trim(),
    include_str!(\"%ROOT%/tests/%PREFIX%.exp\").trim(),
  );

  let analysis = checks::Analysis::from_recipe(&s, &recipe);
  assert_eq(
    &format!(\"{:#?}\", Printable {
      value: &analysis,
      state: &s,
    }).trim(),
    include_str!(\"%ROOT%/tests/%PREFIX%.analysis\").trim(),
  );

  let mut buf = Vec::new();
  analysis.debug_problems(&mut buf, &s).unwrap();
  assert_eq(
    std::str::from_utf8(&buf).unwrap().trim(),
    include_str!(\"%ROOT%/tests/%PREFIX%.problems\").trim(),
  );

  let bt_path = std::path::Path::new(\"%ROOT%/tests/%PREFIX%.backward_tree\");
  if bt_path.exists() {
    let expected = std::fs::read_to_string(bt_path).unwrap();
    let tree = analysis.into_tree().expect(\"Had a valid backward tree test but cannot build a tree\");
    assert_eq(
      &format!(\"{:#?}\", Printable {
        value: &tree,
        state: &s,
      }),
      expected.trim(),
    );

    let table_path = std::path::Path::new(\"%ROOT%/tests/%PREFIX%.raw_table\");
    if table_path.exists() {
      let expected = std::fs::read_to_string(table_path).unwrap();
      let table = render::table::Table::new(&s, &tree);
      assert_eq(
        table.debug().trim(),
        expected.trim(),
      );
    }
  }
}
";

fn main() {
    lalrpop::process_root().unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let destination = Path::new(&out_dir).join("exp_tests.rs");
    let mut test_file = File::create(&destination).unwrap();
    writeln!(test_file, "{}", FILE_PREFIX).unwrap();

    for exp in std::fs::read_dir("tests").unwrap() {
        let exp = exp.unwrap().path().canonicalize().unwrap();
        let fname = exp.file_name().unwrap().to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".apicius") {
            let test = TEST_TEMPLATE
                .replace("%FILE%", &fname)
                .replace("%PREFIX%", prefix)
                .replace("%ROOT%", &manifest_dir);
            writeln!(test_file, "{}", test).unwrap()
        }
    }
}
