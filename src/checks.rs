use std::collections::{BTreeMap, BTreeSet};
use std::io;

pub use crate::types::State;
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
    HasCycle(string_interner::DefaultSymbol),
}

#[derive(Debug)]
pub struct Analysis {
    map: BTreeMap<Option<string_interner::DefaultSymbol>, Vec<Path>>,
    problems: Vec<Problem>,
}

#[derive(Debug)]
pub struct BackwardTree {
    pub actions: Vec<ActionStep>,
    pub paths: Vec<BackwardTree>,
    pub ingredients: Vec<IngredientRef>,
    pub size: usize,
}

impl BackwardTree {
    pub fn debug(&self, w: &mut impl io::Write, s: &State) -> io::Result<()> {
        self.debug_helper(w, s, 0)
    }

    pub fn debug_helper(&self, w: &mut impl io::Write, s: &State, depth: usize) -> io::Result<()> {
        for _ in 0..depth {
            write!(w, "  ")?;
        }
        for a in self.actions.iter() {
            s.debug_action_step(w, a)?;
            write!(w, " -> ")?;
        }
        s.debug_ingredients(w, &self.ingredients)?;
        writeln!(w)?;
        for child in self.paths.iter() {
            child.debug_helper(w, s, depth + 1)?;
        }
        Ok(())
    }
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
                    Problem::HasCycle(sym) => write!(
                        w,
                        "the join point '{}' is involved in a cycle",
                        &state[*sym]
                    )?,
                }
                writeln!(w)?;
            }
        }

        Ok(())
    }

    pub fn find_cycles(&mut self) {
        let mut frontier: Vec<string_interner::DefaultSymbol> = Vec::new();
        let mut seen = BTreeSet::new();

        for path in self.map[&None].iter() {
            if let Input::Join { point } = path.start {
                frontier.push(point.value)
            }
        }

        while let Some(elem) = frontier.pop() {
            if seen.contains(&elem) {
                self.problems.push(Problem::HasCycle(elem));
                break;
            }
            seen.insert(elem);
            for path in self.map[&Some(elem)].iter() {
                if let Input::Join { point } = path.start {
                    frontier.push(point.value);
                }
            }
        }
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
        } else {
            analysis.find_cycles();
        }

        analysis
    }

    fn to_tree_helper(&mut self, path: Path, vec: &mut Vec<BackwardTree>) -> usize {
        let mut size = 0;
        let mut children = Vec::new();
        let mut ingredients;
        match path.start {
            Input::Ingredients { list } => {
                size = list.len();
                ingredients = list;
            }
            Input::Join { point } => {
                ingredients = Vec::new();
                let paths = self.map.remove(&Some(point.value)).unwrap();
                for path in paths.into_iter() {
                    size += self.to_tree_helper(path, &mut children);
                }
            }
        }
        vec.push(BackwardTree {
            paths: children,
            actions: path.actions,
            ingredients: ingredients,
            size: size,
        });
        size
    }

    pub fn into_tree(mut self) -> BackwardTree {
        let mut b = BackwardTree {
            paths: vec![],
            actions: Vec::new(),
            ingredients: Vec::new(),
            size: 0,
        };
        let paths = self.map.remove(&None).unwrap();
        for path in paths.into_iter() {
            b.size += self.to_tree_helper(path, &mut b.paths);
        }
        b
    }
}
