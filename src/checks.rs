use std::collections::BTreeMap;
use std::io;

use crate::types::*;

#[derive(Debug)]
struct Path {
    actions: Vec<ActionStep>,
    start: Input,
}

#[derive(Debug)]
enum Problem {
    NoDone,
    DanglingSteps(Vec<ActionStep>, Input),
}

#[derive(Debug)]
pub struct Analysis {
    map: BTreeMap<Option<string_interner::DefaultSymbol>, Vec<Path>>,
    problems: Vec<Problem>,
}

impl Analysis {
    fn add(&mut self, key: Option<StringRef>, value: Path) {
        self.map
            .entry(key.map(|x| *x))
            .or_insert_with(Vec::new)
            .push(value);
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

    pub fn debug_problems(&self, w: &mut impl io::Write, state: &State) -> io::Result<()> {
        if self.problems.is_empty() {
            writeln!(w, "graph ok")?;
        } else {
            writeln!(w, "graph problems:")?;
            for p in self.problems.iter() {
                write!(w, " - ")?;
                match p {
                    Problem::NoDone => write!(w, "no `<>` state")?,
                    Problem::DanglingSteps(actions, Input::Ingredients { list }) => {
                        write!(w, "path starting from ingredients list '")?;
                        state.debug_ingredients(w, list)?;
                        write!(w, "' goes through actions '")?;
                        for a in actions.iter() {
                            state.debug_action_step(w, a)?;
                        }
                        write!(w, "' but never reaches a join point")?;
                    }
                    Problem::DanglingSteps(actions, Input::Join { point }) => {
                        write!(w, "path starting at join point '{}'", &state[*point])?;
                        write!(w, " goes through action path '")?;
                        for a in actions.iter() {
                            state.debug_action_step(w, a)?;
                            write!(w, " -> ")?;
                        }
                        write!(w, "...' but never reaches a join point")?;
                    }
                }
                writeln!(w)?;
            }
        }

        Ok(())
    }

    pub fn from_recipe(state: &State, recipe: &Recipe) -> Self {
        let mut analysis = Analysis {
            map: BTreeMap::new(),
            problems: Vec::new(),
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

            if !path.actions.is_empty() {
                // we've got leftover actions we haven't put somewhere, which is not great!
                analysis
                    .problems
                    .push(Problem::DanglingSteps(path.actions, path.start));
            }
        }

        if !analysis.map.contains_key(&None) {
            analysis.problems.push(Problem::NoDone);
        }

        analysis
    }
}

pub fn to_tree(state: &State, recipe: &Recipe) -> Result<(), String> {
    let analysis = Analysis::from_recipe(state, recipe);

    analysis.debug(&mut io::stdout(), state).unwrap();
    analysis.debug_problems(&mut io::stdout(), state).unwrap();

    Ok(())
}
