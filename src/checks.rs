//! The `checks` module contains passes which verify and analyze
//! recipes before we render them.

use std::cmp::max;
use std::collections::{BTreeMap, BTreeSet};
use std::{fmt,io};

pub use crate::types::State;
use crate::types::*;

#[derive(Debug)]
struct Path {
    actions: Vec<ActionStep>,
    start: Input,
}

/// A `Problem` represents an invariant failure which would prevent
/// our rendering code from rending a recipe.
///
/// TODO:
///  - Better display of cycles
///  - Discovering disconnected parts of the graph
#[derive(Debug)]
pub enum Problem {
    /// Every recipe needs a `<>` so we can work backwards from it, so
    /// it's an error to omit the `<>` value
    NoDone,
    /// If the last thing in a sequence of actions is not a join point
    /// or `<>`, then those actions are effectively useless
    ///
    /// TODO: detect this after `<>` and not just after join points
    DanglingSteps(Vec<ActionStep>, Input),
    /// We want our recipes to be strictly tree-shaped, so disallow
    /// any cycles. We might lift this restriction in the future, but
    /// it's a _huge_ simplifying assumption for recipe graphing.
    HasCycle(string_interner::DefaultSymbol),
}

/// An `Analysis` takes the raw recipe and turns it into an abstract
/// format more conducive to analysis.
///
/// **NOTE**: this is only public in the API right now for
/// development. This'll eventually be a fully-internal intermediate
/// state between the raw input and the backwards tree.
///
/// Internal note: an `Analysis` contains two things: a map
/// representation of the recipe _and_ a list of invariant
/// violations. The map representation uses join points as keys and
/// maps them to a set of sequences which lead to that join point. For
/// example, given the Apicius source
///
/// ```apicius
/// sample {
///   one -> foo -> $a;
///   two -> bar -> $a;
///   $a -> baz -> <>;
///   three -> quux -> <>;
/// }
/// ```
///
/// we'll end up with a `map` that looks like
///
/// ```yaml
/// Some('$a'):
///   - input: one
///     steps: ['foo']
///   - input: two
///     steps: ['bar']
/// None:
///   - input: '$a'
///     steps: ['baz']
///   - input: three
///     steps: ['quux']
/// ```
#[derive(Debug)]
pub struct Analysis {
    map: BTreeMap<Option<string_interner::DefaultSymbol>, Vec<Path>>,
    problems: Vec<Problem>,
}

/// The "backwards" version of a recipe starting from the end,
/// suitable for rendering.
///
/// This is a recursive tree structure: all join points have been
/// fully removed. That means that in order to produce a
/// `BackwardTree` we'll have to have successfully completed an
/// analysis which shows the graph is valid.
///
/// Each intermediate node contains at least one "action", possibly
/// more, and zero or more ingredients, but the root node will always
/// have zero actions and zero ingredients. (It SHOULD be the only
/// such node.) We also should only ever have ingredients if we're a
/// leaf node (i.e. a node with no children.)
///
/// The 'size' parameter here corresponds to how many distinct input
/// lines lead into it, computed recursively. The `max_depth`
/// parameter corresponds to the longest number of steps which lead
/// into that node plus the number of actions in that node. Both of
/// these are relevant for drawing code.
///
/// For example, for this example graph
///
/// ```apicius
/// sample {
///   one -> foo -> $a;
///   two -> bar -> $a;
///   $a -> baz -> <>;
///   three -> quux -> <>;
/// }
/// ```
///
/// we'll end up with a `BackwardTree` that looks like this, omitting fields that are empty:
///
/// ```yaml
/// size: 3
/// max_depth: 2
/// children:
/// - size: 2
///   max_depth: 2
///   actions: [baz]
///   children:
///   - size: 1
///     max_depth: 1
///     ingredients: [one]
///     actions: [foo]
///   - size: 1
///     max_depth: 1
///     ingredients: [two]
///     actions: [bar]
/// - size: 1
///   max_depth: 1
///   ingredients: [three]
///   actions: [quux]
/// ```
#[derive(Debug)]
pub struct BackwardTree {
    pub actions: Vec<ActionStep>,
    pub paths: Vec<BackwardTree>,
    pub ingredients: Vec<IngredientRef>,
    pub size: usize,
    pub max_depth: usize,
}

impl Analysis {
    /// Add a new `Path` that leads to a given join point (or the root
    /// for `None`)
    fn add(&mut self, key: Option<StringRef>, value: Path) {
        self.map
            .entry(key.map(|x| *x))
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Print the `Analysis` to the provided writer
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

    /// Print the list of `Problem` values for this `Analysis` to the
    /// given writer
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

    /// Find all cycles in the graph.
    /// TODO: also find disconnected components here
    /// TODO: print more of the cycle to make it easier to diagnose,
    /// instead of just, "Hey, here's a node that's involved in a
    /// cycle."
    fn find_cycles(&mut self) {
        // this is just doing DFS with an explicit stack
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

    /// Take a `Recipe` and produce an `Analysis` value from it. This
    /// will still produce an `Analysis` even if there are problems
    /// found with it, but any `Analysis` that has non-zero problems
    /// cannot be turned into a `BackwardTree`.
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

    fn convert_tree_helper(&mut self, path: Path, vec: &mut Vec<BackwardTree>) -> (usize, usize) {
        let mut size = 0;
        let mut children = Vec::new();
        let ingredients;
        let mut max_depth = 0;
        match path.start {
            Input::Ingredients { list } => {
                size = list.len();
                ingredients = list;
            }
            Input::Join { point } => {
                ingredients = Vec::new();
                let paths = self.map.remove(&Some(point.value)).unwrap();
                for path in paths.into_iter() {
                    let (ns, nd) = self.convert_tree_helper(path, &mut children);
                    size += ns;
                    max_depth = max(max_depth, nd);
                }
            }
        }
        max_depth += path.actions.len();
        vec.push(BackwardTree {
            paths: children,
            actions: path.actions,
            ingredients,
            size,
            max_depth,
        });
        (size, max_depth)
    }

    /// Take an `Analysis` value and convert it into a
    /// `BackwardTree`. This reuses some of the same backing memory
    /// and therefore consumes the `Analysis`. If this can't be turned
    /// into a `BackwardTree`, then this will instead return the
    /// vector of problems with it
    pub fn into_tree(mut self) -> Result<BackwardTree, Vec<Problem>> {
        if !self.problems.is_empty() {
            return Err(self.problems);
        }

        let mut b = BackwardTree {
            paths: vec![],
            actions: Vec::new(),
            ingredients: Vec::new(),
            size: 0,
            max_depth: 0,
        };
        let paths = self.map.remove(&None).unwrap();
        for path in paths.into_iter() {
            let (ns, nd) = self.convert_tree_helper(path, &mut b.paths);
            b.size += ns;
            b.max_depth = max(b.max_depth, nd);
        }
        Ok(b)
    }
}

impl<'a> fmt::Debug for Printable<'a, BackwardTree> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = f.debug_struct("BackwardTree");
        str.field("sz", &self.value.size);
        str.field("max_depth", &self.value.max_depth);
        if !self.value.actions.is_empty() {
            str.field("actions", &self.from_seq(&self.value.actions));
        }
        if !self.value.ingredients.is_empty() {
            str.field("ingredients", &self.from_seq(&self.value.ingredients));
        }
        if !self.value.paths.is_empty() {
            str.field("paths", &self.from_seq(&self.value.paths));
        }
        str.finish()
    }
}
