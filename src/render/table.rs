use crate::checks::BackwardTree;
use crate::types::State;

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
        buf.push_str("<table border='1'>");
        for row in self.compute_helper(&self.bt) {
            buf.push_str("<tr>");
            for cell in row {
                buf.push_str(&cell);
            }
            buf.push_str("</tr>");
        }
        buf.push_str("</table>");
        buf
    }

    pub fn compute_helper(&self, focus: &'a BackwardTree) -> Vec<Vec<String>> {
        println!("===");
        focus.debug(&mut std::io::stdout(), self.state);
        println!("===");
        let mut vec = Vec::new();
        let mut first = true;
        for i in focus.ingredients.iter() {
            let mut buf = Vec::new();
            self.state
                .debug_ingredient(&mut buf, &self.state[*i])
                .unwrap();
            let elem = format!("<td>{}</td>", std::str::from_utf8(&buf).unwrap());
            vec.push(vec![elem]);
        }

        if focus.paths.len() == 0 {
            for a in focus.actions.iter() {
                let mut buf = Vec::new();
                self.state.debug_action_step(&mut buf, a);
                let action_str = std::str::from_utf8(&buf).unwrap();
                vec[0].push(format!(
                    "<td rowspan=\"{}\"/>{}</td>",
                    focus.size, action_str
                ));
            }
        }

        for path in focus.paths.iter() {
            for mut row in self.compute_helper(&path) {
                if first {
                    let mut buf = Vec::new();
                    for a in focus.actions.iter() {
                        self.state.debug_action_step(&mut buf, a);
                    }
                    let action_str = std::str::from_utf8(&buf).unwrap();
                    row.push(format!(
                        "<td rowspan=\"{}\"/>{}</td>",
                        focus.size, action_str
                    ));
                    first = false;
                }
                vec.push(row);
            }
        }
        vec
    }
}
