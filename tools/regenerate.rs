use apicius::{checks, grammar, types::{Printable, State}};
use std::io::Write;

fn main() {
    for exp in std::fs::read_dir("tests").unwrap() {
        let exp = exp.unwrap().path().canonicalize().unwrap();
        let fname = exp.file_name().unwrap().to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".apicius") {
            println!("regenerating {}.apicius", prefix);
            let exp_filename = |new_suffix| {
                let mut f = exp.clone();
                f.pop();
                f.push(format!("{}.{}", prefix, new_suffix));
                f
            };

            let src = std::fs::read_to_string(&exp).unwrap();
            let mut state = State::new();
            if let Ok(recipe) = grammar::RecipeParser::new().parse(&mut state, &src) {
                let mut f = std::fs::File::create(exp_filename("exp")).unwrap();
                state.debug_recipe(&mut f, &recipe).unwrap();

                let mut f = std::fs::File::create(exp_filename("analysis")).unwrap();
                let a = checks::Analysis::from_recipe(&state, &recipe);
                a.debug(&mut f, &state).unwrap();

                let mut f = std::fs::File::create(exp_filename("problems")).unwrap();
                a.debug_problems(&mut f, &state).unwrap();

                let bt_path = exp_filename("backward_tree");
                if let Ok(tree) = a.into_tree() {
                    let mut f = std::fs::File::create(bt_path).unwrap();
                    write!(f, "{:#?}", Printable {
                        state: &state,
                        value: &tree,
                    }).unwrap();
                } else if bt_path.exists() {
                    std::fs::remove_file(bt_path).unwrap();
                }
            }
        }
    }
}
