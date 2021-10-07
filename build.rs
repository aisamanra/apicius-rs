use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const FILE_PREFIX: &str = "
use crate::types::*;
use crate::grammar;
";

const TEST_TEMPLATE: &str = "
// test for %FILE%
#[test]
fn test_%PREFIX%() {
  let source = include_str!(%PATH%);
  let mut s = State::new();
  let recipe = grammar::RecipeParser::new().parse(&mut s, source);
  assert!(recipe.is_ok());
  %EXPECTATION%
}
";

const EXP_TEMPLATE: &str = "
  let exp = std::fs::read_to_string(%PATH%).unwrap();
  let mut buf = Vec::new();
  s.debug_recipe(&mut buf, recipe.unwrap()).unwrap();
  assert_eq!(std::str::from_utf8(&buf).unwrap().trim(), exp.trim());
";

fn main() {
    lalrpop::process_root().unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("exp_tests.rs");
    let mut test_file = File::create(&destination).unwrap();
    writeln!(test_file, "{}", FILE_PREFIX).unwrap();
    for exp in std::fs::read_dir("tests").unwrap() {
        let exp = exp.unwrap().path().canonicalize().unwrap();
        let fname = exp.file_name().unwrap().to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".apicius") {
            let mut expected = exp.clone();
            expected.pop();
            expected.push(format!("{}.exp", prefix));

            let mut test = TEST_TEMPLATE
                .replace("%FILE%", &fname)
                .replace("%PREFIX%", prefix)
                .replace("%PATH%", &format!("{:?}", exp.as_path()));
            if expected.exists() {
                test = test.replace(
                    "%EXPECTATION%",
                    &EXP_TEMPLATE.replace("%PATH%", &format!("{:?}", expected.as_path())),
                );
            } else {
                test = test.replace("%EXPECTATION%", "");
            }
            writeln!(test_file, "{}", test).unwrap()
        }
    }
}
