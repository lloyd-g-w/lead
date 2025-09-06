use std::collections::{HashMap, HashSet};

use crate::{
    cell::{Cell, CellRef},
    evaluator::{Eval, evaluate},
    tokenizer::Literal,
};

pub struct Grid {
    cells: HashMap<CellRef, Cell>,
}

impl Grid {
    pub fn new() -> Grid {
        return Grid {
            cells: HashMap::new(),
        };
    }
}

impl Grid {
    pub fn set_cell(&mut self, cell_ref: CellRef, raw_val: String) -> Result<Eval, String> {
        if self.cells.contains_key(&cell_ref) && self.cells[&cell_ref].raw() == raw_val {
            return self.get_cell(cell_ref);
        }

        let eval: Eval;
        let deps: HashSet<CellRef>;

        if let Some(c) = raw_val.chars().nth(0)
            && c == '='
        {
            (eval, deps) = evaluate(raw_val[1..].to_owned(), Some(&self))?;
            // for dep in deps {}
        } else {
            match evaluate(raw_val.to_owned(), Some(&self)) {
                Ok(e) => {
                    (eval, deps) = e;
                }
                Err(_) => eval = Eval::Literal(Literal::String(raw_val.to_owned())),
            }
        }

        self.cells
            .insert(cell_ref, Cell::new(eval.clone(), raw_val));
        Ok(eval)
    }

    // pub fn get_cell(&mut self, cell_ref: CellRef) -> Result<(String, Eval), String> {
    pub fn get_cell(&self, cell_ref: CellRef) -> Result<Eval, String> {
        if !self.cells.contains_key(&cell_ref) {
            return Err(format!("Cell at {:?} not found.", cell_ref));
        }

        let cell = &self.cells[&cell_ref];

        // Ok((cell.raw(), cell.eval()))
        Ok(cell.eval())
    }

    pub fn add_cell_dep(&mut self, cell_ref: CellRef, dep_ref: CellRef) -> Result<(), String> {
        if !self.cells.contains_key(&cell_ref) {
            return Err(format!("Cell at {:?} not found.", cell_ref));
        }

        if let Some(cell) = self.cells.get_mut(&cell_ref) {
            cell.add_i_dep(dep_ref);
        }

        Ok(())
    }
}
