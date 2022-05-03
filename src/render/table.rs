use crate::checks::BackwardTree;
use crate::types::{ActionStep, IngredientRef, State};

pub struct TableGenerator<'a> {
    state: &'a State,
    bt: &'a BackwardTree,
}

impl<'a> TableGenerator<'a> {
    pub fn new(state: &'a State, bt: &'a BackwardTree) -> TableGenerator<'a> {
        TableGenerator { state, bt }
    }

    pub fn compute(&self) -> String {
        let mut buf = String::new();
        buf.push_str("<table border='1'>\n");
        for row in self.compute_helper(self.bt, self.bt.max_depth) {
            buf.push_str("  <tr>");
            for cell in row {
                buf.push_str(&cell);
            }
            buf.push_str("</tr>\n");
        }
        buf.push_str("</table>\n");
        buf
    }

    fn render_ingredient(&self, i: IngredientRef) -> String {
        let i = &self.state[i];
        if let Some(amt) = i.amount {
            format!(
                "<span class=\"amt\">{}</span> {}",
                &self.state[amt], &self.state[i.stuff]
            )
        } else {
            self.state[i.stuff].to_string()
        }
    }

    fn render_action(&self, a: &ActionStep) -> String {
        let mut buf = String::new();
        buf.push_str(&self.state[a.action]);
        if !a.seasonings.is_empty() {
            buf.push_str("<div class=\"seasonings\">+");
            for i in a.seasonings.iter() {
                buf.push_str(&self.render_ingredient(*i));
                buf.push(' ');
            }
            buf.push_str("</div>");
        }
        buf
    }

    fn compute_helper(&self, focus: &'a BackwardTree, depth: usize) -> Vec<Vec<String>> {
        let mut vec = Vec::new();
        let mut first = true;
        let colspan = depth - focus.max_depth + 1;

        for i in focus.ingredients.iter() {
            let elem = format!(
                "<td class=\"ingredient\" colspan=\"{}\">{}</td>",
                colspan - 1,
                self.render_ingredient(*i),
            );
            vec.push(vec![elem]);
        }

        if focus.paths.is_empty() {
            for a in focus.actions.iter() {
                vec[0].push(format!(
                    "<td rowspan=\"{}\" class=\"action\">{}</td>",
                    focus.size,
                    self.render_action(a)
                ));
            }
        }

        for path in focus.paths.iter() {
            for mut row in self.compute_helper(path, focus.max_depth - focus.actions.len() + 1) {
                if first {
                    if focus.actions.is_empty() && focus.ingredients.is_empty() {
                        row.push(format!(
                            "<td rowspan=\"{}\" class=\"done\"></td>",
                            focus.size
                        ));
                    } else {
                        for a in focus.actions.iter() {
                            row.push(format!(
                                "<td rowspan=\"{}\">{}</td>",
                                focus.size,
                                self.render_action(a)
                            ))
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
