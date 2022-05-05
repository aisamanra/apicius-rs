use apicius::{
    checks,
    grammar,
    render::table::Table,
    types::{State, ToPrintable},
};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for exp in std::fs::read_dir("tests")? {
        let exp = exp?.path().canonicalize()?;
        let fname = exp.file_name().unwrap().to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".apicius") {
            println!("regenerating {}.apicius", prefix);
            let exp_filename = |new_suffix| {
                let mut f = exp.clone();
                f.pop();
                f.push(format!("{}.{}", prefix, new_suffix));
                f
            };

            let src = std::fs::read_to_string(&exp)?;
            let mut state = State::new();
            if let Ok(recipe) = grammar::RecipeParser::new().parse(&mut state, &src) {
                let mut f = std::fs::File::create(exp_filename("exp"))?;
                state.debug_recipe(&mut f, &recipe)?;

                let mut f = std::fs::File::create(exp_filename("analysis"))?;
                let a = checks::Analysis::from_recipe(&state, &recipe);
                write!(f, "{:#?}", a.printable(&state))?;

                let mut f = std::fs::File::create(exp_filename("problems"))?;
                a.debug_problems(&mut f, &state)?;

                let bt_path = exp_filename("backward_tree");
                if let Ok(tree) = a.into_tree() {
                    {
                        let mut f = std::fs::File::create(bt_path)?;
                        write!(f, "{:#?}", tree.printable(&state))?;
                    }

                    {
                        let table = Table::new(&state, &tree);
                        let mut f = std::fs::File::create(exp_filename("raw_table"))?;
                        write!(f, "{}", table.debug())?;
                    }
                } else if bt_path.exists() {
                    std::fs::remove_file(bt_path)?;
                }
            }
        }
    }

    Ok(())
}
