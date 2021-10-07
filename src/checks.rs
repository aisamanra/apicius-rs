use std::collections::BTreeMap;
use std::io;

use crate::types::*;

#[derive(Debug)]
struct Path {
    actions: Vec<ActionStep>,
    start: Input,
}

#[derive(Debug)]
pub struct Analysis {
    map: BTreeMap<Option<StringRef>, Vec<Path>>,
}

impl Analysis {
    fn add(&mut self, key: Option<StringRef>, value: Path) {
        self.map.entry(key).or_insert_with(Vec::new).push(value);
    }

    pub fn debug(&self, w: &mut impl io::Write, state: &State) -> io::Result<()> {
        writeln!(w, "analysis {{")?;
        for (k, v) in self.map.iter() {
            if let Some(name) = k {
                writeln!(w, "  {}", &state[*name])?;
            } else {
                writeln!(w, "  DONE")?;
            }
            for alt in v.iter() {
                write!(w, "    ")?;
                for a in alt.actions.iter() {
                    write!(w, " <- {}", &state[a.action])?;
                }
                write!(w, " <- ")?;
                state.debug_input(w, &alt.start).unwrap();
                writeln!(w)?;
            }
        }
        writeln!(w, "}}")
    }

    pub fn from_recipe(state: &State, recipe: &Recipe) -> Self {
        let mut analysis = Analysis {
            map: BTreeMap::new(),
        };

        'outer: for rule in recipe.rules.iter() {
            let rule = &state[*rule];
            let mut path = Path {
                actions: Vec::new(),
                start: rule.input.clone(),
            };
            for action in rule.actions.iter() {
                match action {
                    Action::Action { step } => path.actions.push(step.clone()),
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

        analysis
    }
}

pub fn to_tree(state: &State, recipe: &Recipe) -> Result<(), String> {
    let analysis = Analysis::from_recipe(state, recipe);

    analysis.debug(&mut io::stdout(), state).unwrap();
    Ok(())
}
