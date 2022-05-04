use std::ops::{Deref, Index};
use std::{fmt, io};

// A wrapper struct that indicates where a given value was positioned
// in the
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc<T> {
    pub l: usize,
    pub r: usize,
    pub value: T,
}

impl<T> Deref for Loc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub type StringRef = Loc<string_interner::DefaultSymbol>;

/// A recipe as written is a name and a set of rules.
#[derive(Debug)]
pub struct Recipe {
    pub name: StringRef,
    pub rules: Vec<RuleRef>,
}

/// A rule starts from an input and includes a sequence of actions
/// afterwards. No invariant-checking has been performed on values of
/// type `Rule`, so it's possible for it to represent recipes which
/// don't adhere to broader rules.
#[derive(Debug)]
pub struct Rule {
    pub input: Input,
    pub actions: Vec<Action>,
}

/// Each step of an action consists of the thing being done along with
/// an optional set of added ingredients
#[derive(Debug, Clone)]
pub struct ActionStep {
    pub action: StringRef,
    pub seasonings: Vec<IngredientRef>,
}

/// A step can be one of three things: an action, a join point, or the
/// special `<>` symbol to represent a finished recipe.
#[derive(Debug)]
pub enum Action {
    Action { step: ActionStep },
    Join { point: StringRef },
    Done,
}

/// The start of a rule can be either a list of ingredients or a join
/// point
#[derive(Debug, Clone)]
pub enum Input {
    Ingredients { list: Vec<IngredientRef> },
    Join { point: StringRef },
}

/// An ingredient is an optional specified amount as well as the name
/// of the ingredient
#[derive(Debug)]
pub struct Ingredient {
    pub amount: Option<StringRef>,
    pub stuff: StringRef,
}

/// Ingredients are stored in a packed array, and rules will in turn
/// reference them by index
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IngredientRef {
    idx: usize,
}

/// Rules are stored in a packaged array, and recipes will in turn
/// reference them by index
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RuleRef {
    idx: usize,
}

/// A `State` value contains the packed arrays used to represent a
/// recipe
#[derive(Debug)]
pub struct State {
    ingredients: Vec<Ingredient>,
    rules: Vec<Rule>,
    strings: string_interner::StringInterner,
}

impl State {
    /// Create a new `State` value with no existing state
    pub fn new() -> State {
        State {
            ingredients: Vec::new(),
            rules: Vec::new(),
            strings: string_interner::StringInterner::new(),
        }
    }

    /// Create a new `Ingredient` value and pack it, returning the
    /// index. We never delete existing ingredients, so an
    /// `IngredientRef` is guaranteed to reference a valid
    /// `Ingredient`.
    pub fn add_ingredient(&mut self, i: Ingredient) -> IngredientRef {
        let idx = self.ingredients.len();
        self.ingredients.push(i);
        IngredientRef { idx }
    }

    /// Create a new `Rule` value and pack it, returning the index. We
    /// never delete existing rules, so a `RuleRef` is guaranteed to
    /// reference a valid `Rule`.
    pub fn add_rule(&mut self, r: Rule) -> RuleRef {
        let idx = self.rules.len();
        self.rules.push(r);
        RuleRef { idx }
    }

    /// Intern a string or, if the string has been interned before,
    /// grab the existing one. This means that comparison of two
    /// strings is an integer comparison instead of a full string
    /// comparison.
    pub fn add_string(&mut self, s: &str) -> string_interner::DefaultSymbol {
        self.strings.get_or_intern(s)
    }

    /// Print an `Ingredient` to a writer
    pub fn debug_ingredient(&self, w: &mut impl io::Write, i: &Ingredient) -> io::Result<()> {
        if let Some(amt) = i.amount {
            write!(w, "[{}] ", &self[amt])?;
        }
        write!(w, "{}", &self[i.stuff])
    }

    /// Print a sequence of `Ingredient`s to a writer
    pub fn debug_ingredients(
        &self,
        w: &mut impl io::Write,
        list: &[IngredientRef],
    ) -> io::Result<()> {
        if list.is_empty() {
            return Ok(());
        }

        self.debug_ingredient(w, &self[list[0]])?;
        for i in list.iter().skip(1) {
            write!(w, " + ")?;
            self.debug_ingredient(w, &self[*i])?;
        }
        Ok(())
    }

    /// Print an `Input` to a writer
    pub fn debug_input(&self, w: &mut impl io::Write, i: &Input) -> io::Result<()> {
        match i {
            Input::Join { point } => write!(w, "{}", &self[*point])?,
            Input::Ingredients { list } => self.debug_ingredients(w, list)?,
        }
        Ok(())
    }

    /// Print an `ActionStep` to a writer
    pub fn debug_action_step(&self, w: &mut impl io::Write, a: &ActionStep) -> io::Result<()> {
        write!(w, "{}", &self[a.action])?;
        if !a.seasonings.is_empty() {
            write!(w, " & ")?;
            self.debug_ingredients(w, &a.seasonings)?;
        }
        Ok(())
    }

    /// Print an `Action` to a writer
    pub fn debug_action(&self, w: &mut impl io::Write, a: &Action) -> io::Result<()> {
        match a {
            Action::Action {
                step: ActionStep { action, seasonings },
            } => {
                write!(w, "{}", &self[*action])?;
                if !seasonings.is_empty() {
                    write!(w, " & ")?;
                    self.debug_ingredients(w, seasonings)?;
                }
            }
            Action::Join { point } => write!(w, "{}", &self[*point])?,
            Action::Done => write!(w, "<>")?,
        }
        Ok(())
    }

    /// Print a `Recipe` to a writer
    pub fn debug_recipe(&self, w: &mut impl io::Write, r: &Recipe) -> io::Result<()> {
        writeln!(w, "{} {{", self.strings.resolve(*r.name).unwrap())?;
        for rule in r.rules.iter() {
            let rule = &self[*rule];
            write!(w, "  ")?;
            self.debug_input(w, &rule.input)?;
            for action in rule.actions.iter() {
                write!(w, " -> ")?;
                self.debug_action(w, action)?;
            }
            writeln!(w, ";")?;
        }
        writeln!(w, "}}")
    }
}

// These allow us to use our `*Ref` types and get the appropriate
// value directly out of the state
impl Index<RuleRef> for State {
    type Output = Rule;

    fn index(&self, rf: RuleRef) -> &Self::Output {
        self.rules.index(rf.idx)
    }
}

impl Index<IngredientRef> for State {
    type Output = Ingredient;

    fn index(&self, if_: IngredientRef) -> &Self::Output {
        self.ingredients.index(if_.idx)
    }
}

impl Index<StringRef> for State {
    type Output = str;

    fn index(&self, sf: StringRef) -> &Self::Output {
        self.strings.resolve(*sf).unwrap()
    }
}

impl Index<string_interner::DefaultSymbol> for State {
    type Output = str;

    fn index(&self, sf: string_interner::DefaultSymbol) -> &Self::Output {
        self.strings.resolve(sf).unwrap()
    }
}

impl Default for State {
    fn default() -> State {
        Self::new()
    }
}

pub struct Printable<'a, T> {
    pub state: &'a State,
    pub value: &'a T,
}

impl<'a, T> Printable<'a, T> {
    pub fn from_seq<R>(&self, seq: &'a [R]) -> Vec<Printable<R>> {
        seq.iter()
            .map(|value| Printable {
                state: self.state,
                value,
            })
            .collect()
    }
}

impl<'a> fmt::Debug for Printable<'a, IngredientRef> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Printable {
            state: self.state,
            value: &self.state[*self.value],
        }
        .fmt(f)
    }
}

impl<'a> fmt::Debug for Printable<'a, Ingredient> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(amt) = self.value.amount {
            write!(f, "[{}]", &self.state[amt])?;
        }
        write!(f, "{}", &self.state[self.value.stuff])
    }
}

impl Ingredient {
    pub fn debug<'a>(&'a self, state: &'a State) -> Printable<'a, Ingredient> {
        Printable { value: self, state }
    }
}

impl<'a> fmt::Debug for Printable<'a, ActionStep> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.state[self.value.action])?;
        if !self.value.seasonings.is_empty() {
            write!(f, " & ")?;
            write!(f, "{:?}", self.from_seq(&self.value.seasonings))?;
        }
        Ok(())
    }
}

impl ActionStep {
    pub fn debug<'a>(&'a self, state: &'a State) -> Printable<'a, ActionStep> {
        Printable { value: self, state }
    }
}
