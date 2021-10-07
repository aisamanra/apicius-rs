use std::io;
use std::ops::{Deref, Index};

use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ApiciusError {
    #[error("Recipe does not include `DONE`")]
    MissingDone,
}

pub type StringRef = Loc<string_interner::DefaultSymbol>;

#[derive(Debug)]
pub struct Recipe {
    pub name: StringRef,
    pub rules: Vec<RuleRef>,
}

#[derive(Debug)]
pub struct Rule {
    pub input: Input,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub struct ActionStep {
    pub action: StringRef,
    pub seasonings: Vec<IngredientRef>,
}

#[derive(Debug)]
pub enum Action {
    Action { step: ActionStep },
    Join { point: StringRef },
    Done,
}

#[derive(Debug, Clone)]
pub enum Input {
    Ingredients { list: Vec<IngredientRef> },
    Join { point: StringRef },
}

#[derive(Debug)]
pub struct Ingredient {
    pub amount: Option<StringRef>,
    pub stuff: StringRef,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IngredientRef {
    idx: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RuleRef {
    idx: usize,
}

#[derive(Debug)]
pub struct State {
    ingredients: Vec<Ingredient>,
    rules: Vec<Rule>,
    strings: string_interner::StringInterner,
}

impl State {
    pub fn new() -> State {
        State {
            ingredients: Vec::new(),
            rules: Vec::new(),
            strings: string_interner::StringInterner::new(),
        }
    }

    pub fn add_ingredient(&mut self, i: Ingredient) -> IngredientRef {
        let idx = self.ingredients.len();
        self.ingredients.push(i);
        IngredientRef { idx }
    }

    pub fn add_rule(&mut self, r: Rule) -> RuleRef {
        let idx = self.rules.len();
        self.rules.push(r);
        RuleRef { idx }
    }

    pub fn add_string(&mut self, s: &str) -> string_interner::DefaultSymbol {
        self.strings.get_or_intern(s)
    }

    pub fn debug_ingredient(&self, w: &mut impl io::Write, i: &Ingredient) -> io::Result<()> {
        if let Some(amt) = i.amount {
            write!(w, "[{}] ", &self[amt])?;
        }
        write!(w, "{}", &self[i.stuff])
    }

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

    pub fn debug_input(&self, w: &mut impl io::Write, i: &Input) -> io::Result<()> {
        match i {
            Input::Join { point } => write!(w, "{}", &self[*point])?,
            Input::Ingredients { list } => self.debug_ingredients(w, list)?,
        }
        Ok(())
    }

    pub fn debug_action_step(&self, w: &mut impl io::Write, a: &ActionStep) -> io::Result<()> {
        write!(w, "{}", &self[a.action])?;
        if !a.seasonings.is_empty() {
            write!(w, " & ")?;
            self.debug_ingredients(w, &a.seasonings)?;
        }
        Ok(())
    }

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
