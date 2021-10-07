use std::collections::HashMap;

use crate::types::*;

#[derive(Debug)]
struct Path {
    actions: Vec<ActionStep>,
    start: Input,
}

#[derive(Debug)]
struct Analysis {
    map: HashMap<Option<StringRef>, Vec<Path>>,
}

impl Analysis {
    fn add(&mut self, key: Option<StringRef>, value: Path) {
        self.map.entry(key)
            .or_insert_with(|| Vec::new())
            .push(value);
    }
}

pub fn to_tree(state: &State, recipe: &Recipe) -> Result<(), String> {
    let mut analysis = Analysis { map: HashMap::new() };
    'outer: for rule in recipe.rules.iter() {
        let rule = &state[*rule];
        let mut path = Path {
            actions: Vec::new(),
            start: rule.input.clone(),
        };
        for action in rule.actions.iter() {
            match action {
                Action::Action { step } =>
                    path.actions.push(step.clone()),
                Action::Join { point } => {
                    analysis.add(Some(*point), path);
                    path = Path {
                        actions: Vec::new(),
                        start: Input::Join { point: *point },
                    };
                }
                Action::Done => {
                    analysis.add(None, path);
                    continue 'outer;
                }
            }
        }
    }

    println!("Analysis:");
    for (k, v) in analysis.map.iter() {
        if let Some(name) = k {
            println!("  {}", &state[*name]);
        } else {
            println!("  DONE");
        }
        for alt in v.iter() {
            print!("    ");
            for a in alt.actions.iter() {
                print!(" <- {}", &state[a.action]);
            }
            print!(" <- ");
            state.debug_input(&alt.start);
            println!(";");
        }
    }
    Ok(())
}
