use crate::checks::BackwardTree;
use crate::types::{ActionStep, IngredientRef, State};

#[derive(Debug)]
struct Cell<'a> {
    colspan: usize,
    rowspan: usize,
    contents: CellData<'a>,
}

#[derive(Debug)]
struct CellIngredient<'a> {
    name: &'a str,
    amount: Option<&'a str>,
}

impl<'a> CellIngredient<'a> {
    fn html(&self) -> String {
        if let Some(amt) = self.amount {
            format!("<span class=\"amt\">{}</span> {}", amt, self.name,)
        } else {
            self.name.to_string()
        }
    }
}

#[derive(Debug)]
enum CellData<'a> {
    Ingredient {
        i: CellIngredient<'a>,
    },
    Step {
        name: &'a str,
        seasonings: Vec<CellIngredient<'a>>,
    },
    Done,
}

impl<'a> Cell<'a> {
    fn html(&self) -> String {
        match &self.contents {
            CellData::Done => "<>".to_string(),
            CellData::Step { name, seasonings } => {
                let mut buf = String::new();
                buf.push_str(name);
                if seasonings.is_empty() {
                    return buf;
                }
                buf.push_str("<div class=\"seasonings\">+");
                for i in seasonings.iter() {
                    buf.push_str(&i.html());
                    buf.push(' ');
                }
                buf.push_str("</div>");
                buf
            }
            CellData::Ingredient { i } => i.html(),
        }
    }

    fn html_class(&self) -> &'static str {
        match self.contents {
            CellData::Ingredient { .. } => "ingredient",
            CellData::Step { .. } => "step",
            CellData::Done => "done",
        }
    }
}

pub struct Table<'a> {
    table_data: Vec<Vec<Cell<'a>>>,
}

impl<'a> Table<'a> {
    pub fn new(state: &'a State, bt: &'a BackwardTree) -> Table<'a> {
        Table {
            table_data: TableGenerator::new(state).to_table(bt, bt.max_depth),
        }
    }

    pub fn html(&self) -> String {
        let mut buf = String::new();
        buf.push_str("<table>\n");
        for row in self.table_data.iter() {
            buf.push_str("  <tr>");
            for cell in row.iter() {
                buf.push_str(&format!(
                    "<td class=\"{}\" rowspan=\"{}\" colspan=\"{}\">{}</td>",
                    cell.html_class(),
                    cell.rowspan,
                    cell.colspan,
                    cell.html()
                ));
            }
            buf.push_str("  </tr>\n");
        }
        buf.push_str("</table\n");
        buf
    }
}

struct TableGenerator<'a> {
    state: &'a State,
}

impl<'a> TableGenerator<'a> {
    fn new(state: &'a State) -> TableGenerator<'a> {
        TableGenerator { state }
    }

    fn ingredient_to_cell_ingredient(&self, i: IngredientRef) -> CellIngredient<'a> {
        let i = &self.state[i];
        CellIngredient {
            name: &self.state[i.stuff],
            amount: i.amount.map(|amt| &self.state[*amt]),
        }
    }

    fn action_to_cell(&self, a: &ActionStep) -> CellData<'a> {
        CellData::Step {
            name: &self.state[a.action],
            seasonings: a
                .seasonings
                .iter()
                .map(|i| self.ingredient_to_cell_ingredient(*i))
                .collect(),
        }
    }

    fn to_table(&self, focus: &'a BackwardTree, depth: usize) -> Vec<Vec<Cell<'a>>> {
        let mut vec = Vec::new();
        let mut first = true;

        for i in focus.ingredients.iter() {
            let elem = Cell {
                rowspan: 1,
                colspan: depth - focus.actions.len() + 1,
                contents: CellData::Ingredient {
                    i: self.ingredient_to_cell_ingredient(*i),
                },
            };
            vec.push(vec![elem]);
        }

        if focus.paths.is_empty() {
            for a in focus.actions.iter() {
                vec[0].push(Cell {
                    rowspan: focus.size,
                    colspan: 1,
                    contents: self.action_to_cell(a),
                })
            }
        }

        for path in focus.paths.iter() {
            for mut row in self.to_table(path, depth - focus.actions.len()) {
                if first {
                    if focus.actions.is_empty() && focus.ingredients.is_empty() {
                        row.push(Cell {
                            rowspan: focus.size,
                            colspan: 1,
                            contents: CellData::Done,
                        });
                    } else {
                        for a in focus.actions.iter() {
                            row.push(Cell {
                                rowspan: focus.size,
                                colspan: 1,
                                contents: self.action_to_cell(a),
                            });
                        }
                    }
                    first = false;
                }
                vec.push(row);
            }
        }
        vec
    }
}
