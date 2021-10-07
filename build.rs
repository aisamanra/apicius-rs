use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const FILE_PREFIX: &str = "
use crate::types::*;
use crate::grammar;
use crate::checks;
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
  assert_eq!(
    std::str::from_utf8(&buf).unwrap().trim(),
    include_str!(\"%ROOT%/tests/%PREFIX%.exp\").trim(),
  );

  let mut buf = Vec::new();
  let analysis = checks::Analysis::from_recipe(&s, &recipe);
  analysis.debug(&mut buf, &s).unwrap();
  assert_eq!(
    std::str::from_utf8(&buf).unwrap().trim(),
    include_str!(\"%ROOT%/tests/%PREFIX%.analysis\").trim(),
  );

  let mut buf = Vec::new();
  analysis.debug_problems(&mut buf, &s).unwrap();
  assert_eq!(
    std::str::from_utf8(&buf).unwrap().trim(),
    include_str!(\"%ROOT%/tests/%PREFIX%.problems\").trim(),
  );
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
