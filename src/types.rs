use std::ops::Index;

pub type StringRef = string_interner::DefaultSymbol;

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

#[derive(Debug)]
pub enum Action {
    Action {
        action: StringRef,
        seasonings: Vec<IngredientRef>,
    },
    Join {
        point: StringRef,
    },
    Done,
}

#[derive(Debug)]
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

    pub fn add_string(&mut self, s: &str) -> StringRef {
        self.strings.get_or_intern(s)
    }

    fn debug_ingredient(&self, i: &Ingredient) {
        if let Some(amt) = i.amount {
            print!("[{}] ", &self[amt]);
        }
        print!("{}", &self[i.stuff]);
    }

    fn debug_ingredients(&self, list: &Vec<IngredientRef>) {
        if list.is_empty() {
            return;
        }

        self.debug_ingredient(&self[list[0]]);
        for i in list.iter().skip(1) {
            print!(" + ");
            self.debug_ingredient(&self[*i]);
        }
    }

    fn debug_input(&self, i: &Input) {
        match i {
            Input::Join { point } => print!("{}", &self[*point]),
            Input::Ingredients { list } => self.debug_ingredients(list),
        }
    }

    fn debug_action(&self, a: &Action) {
        match a {
            Action::Action { action, seasonings } => {
                print!("{}", &self[*action]);
                if !seasonings.is_empty() {
                    print!(" & ");
                    self.debug_ingredients(seasonings);
                }
            }
            Action::Join { point } => print!("{}", &self[*point]),
            Action::Done => print!("DONE"),
        }
    }

    pub fn debug_recipe(&self, r: Recipe) {
        println!("{} {{", self.strings.resolve(r.name).unwrap());
        for rule in r.rules {
            let rule = &self[rule];
            print!("  ");
            self.debug_input(&rule.input);
            for action in rule.actions.iter() {
                print!(" -> ");
                self.debug_action(&action);
            }
            println!(";");
        }
        println!("}}");
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
        self.strings.resolve(sf).unwrap()
    }
}
