use std::collections::{HashMap, HashSet};

use log::info;

use crate::{
    cell::{Cell, CellRef},
    common::{LeadErr, LeadErrCode, Literal},
    evaluator::{Eval, evaluate},
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

    pub fn update_cell(
        &mut self,
        cell_ref: CellRef,
        raw_val: String,
    ) -> Result<Vec<CellRef>, String> {
        if self.cells.contains_key(&cell_ref) && self.cells[&cell_ref].raw() == raw_val {
            return Ok(Vec::new());
        }

        let eval: Eval;
        let mut precs: HashSet<CellRef> = HashSet::new();
        let mut updated_cells = vec![cell_ref];

        if raw_val.chars().nth(0) != Some('=') {
            eval = Eval::Literal(Literal::String(raw_val.to_owned()));
        } else {
            // Evaluate raw expr and get precedents
            let (res_eval, res_precs) = evaluate(raw_val[1..].to_owned(), Some(&self));
            eval = res_eval;
            precs = res_precs;
        }

        if self.cells.contains_key(&cell_ref) {
            updated_cells = self
                .update_exisiting_cell(raw_val, eval, precs, cell_ref)
                .into_iter()
                .chain(updated_cells)
                .collect();
        } else {
            self.create_cell(raw_val, eval, precs, cell_ref);
        }

        Ok(updated_cells)
    }

    pub fn quick_eval(&mut self, raw_val: String) -> Eval {
        if raw_val.chars().nth(0) != Some('=') {
            Eval::Literal(Literal::String(raw_val.to_owned()))
        } else {
            let (res_eval, ..) = evaluate(raw_val[1..].to_owned(), Some(&self));
            res_eval
        }
    }

    pub fn get_cell(&self, cell_ref: CellRef) -> Result<Cell, String> {
        if !self.cells.contains_key(&cell_ref) {
            return Err(format!("Cell at {:?} not found.", cell_ref));
        }

        let cell = &self.cells[&cell_ref];

        // Ok((cell.raw(), cell.eval()))
        Ok(cell.to_owned())
    }

    pub fn get_cell_mut(&mut self, cell_ref: CellRef) -> Result<&mut Cell, String> {
        if let Some(res) = self.cells.get_mut(&cell_ref) {
            Ok(res)
        } else {
            Err(format!("Cell at {:?} not found.", cell_ref))
        }
    }

    // This is a topological order on the precedents graph
    // i.e. if a requires b (e.g. a = 1 + b) then a -> b
    // so a comes before b in the topo order
    fn topo_order(&self, from: CellRef) -> (Vec<CellRef>, bool) {
        let mut res = Vec::new();
        let mut temp = HashSet::new();
        let mut perm = HashSet::new();
        let mut cycle_detected = false;

        self.topo_visit(
            from,
            &mut temp,
            &mut perm,
            &mut res,
            &mut cycle_detected,
            from,
        );

        (res, cycle_detected)
    }

    fn topo_visit(
        &self,
        cell: CellRef,
        temp: &mut HashSet<CellRef>,
        perm: &mut HashSet<CellRef>,
        res: &mut Vec<CellRef>,
        cycle_detected: &mut bool,
        caller: CellRef,
    ) {
        if perm.contains(&cell) {
            return;
        }

        if !temp.insert(cell) {
            *cycle_detected = true;
            return;
        }

        if !self.cells.contains_key(&cell) {
            perm.insert(cell);
            res.push(cell);
            return;
        }

        // Walk dependencies if this cell exists; otherwise treat as leaf/external ref.
        if let Some(cell_data) = self.cells.get(&cell) {
            for &dep in cell_data.deps().iter() {
                self.topo_visit(dep, temp, perm, res, cycle_detected, caller);
            }
        }

        // Done exploring this node
        temp.remove(&cell);
        perm.insert(cell);
        if cell != caller {
            res.push(cell);
        }
    }

    fn update_exisiting_cell(
        &mut self,
        raw: String,
        new_eval: Eval,
        new_precs: HashSet<CellRef>,
        cell_ref: CellRef,
    ) -> Vec<CellRef> {
        let (old_precs, old_eval) = match self.cells.get_mut(&cell_ref) {
            Some(cell) => {
                cell.set_raw(raw);
                (cell.precs().clone(), cell.eval().clone())
            }
            None => return Vec::new(),
        };

        // diffs (outside any borrow)
        let removed: Vec<_> = old_precs.difference(&new_precs).cloned().collect(); // old \ new
        let added: Vec<_> = new_precs.difference(&old_precs).cloned().collect(); // new \ old
        let eval_changed = old_eval != new_eval;

        // ---- phase 2: apply (fresh borrows) ----
        for p in &removed {
            if let Some(c) = self.cells.get_mut(p) {
                c.remove_dep(&cell_ref);
            }
        }
        for p in &added {
            if let Some(c) = self.cells.get_mut(p) {
                c.add_dep(cell_ref);
            } else {
                self.cells.insert(
                    *p,
                    Cell::new_all(
                        *p,
                        Eval::Unset,
                        "".into(),
                        HashSet::new(),
                        HashSet::from([cell_ref]),
                    ),
                );
            }
        }

        let cell = self.cells.get_mut(&cell_ref).unwrap(); // Should be impossible to crash
        cell.set_precs(new_precs);

        if eval_changed {
            cell.set_eval(new_eval);
            match self.propagate(cell_ref) {
                Ok(affected_cells) => affected_cells,
                Err(affected_cells) => {
                    let cell = self.cells.get_mut(&cell_ref).unwrap();
                    cell.set_eval(Eval::Err(LeadErr {
                        title: "Propagation error.".into(),
                        desc: "Circular dependencies detected.".into(),
                        code: LeadErrCode::Ref,
                    }));

                    affected_cells
                }
            }
        } else {
            Vec::new()
        }
    }

    fn create_cell(&mut self, raw: String, eval: Eval, precs: HashSet<CellRef>, cell_ref: CellRef) {
        for prec in &precs {
            if let Some(it) = self.cells.get_mut(&prec) {
                it.add_dep(cell_ref);
            } else {
                self.cells.insert(
                    *prec,
                    Cell::new_all(
                        *prec,
                        Eval::Unset,
                        "".into(),
                        HashSet::new(),
                        HashSet::from([cell_ref]),
                    ),
                );

                info!("{:?}", self.cells.get(&prec));
            }
        }

        self.cells.insert(
            cell_ref,
            Cell::new_all(cell_ref, eval, raw, precs, HashSet::new()),
        );
    }

    fn propagate(&mut self, from: CellRef) -> Result<Vec<CellRef>, Vec<CellRef>> {
        let (topo, cycle_detected) = self.topo_order(from);

        if !cycle_detected {
            for cell_ref in topo.to_owned() {
                let raw = if let Some(cell) = self.cells.get(&cell_ref) {
                    let s = cell.raw();
                    if let Some(rest) = s.strip_prefix('=') {
                        rest.to_owned()
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let (e, _) = evaluate(raw, Some(self));

                if let Some(cell) = self.cells.get_mut(&cell_ref) {
                    cell.set_eval(e);
                }
            }
            Ok(topo)
        } else {
            let err = LeadErr {
                title: "Propagation error.".into(),
                desc: "Circular dependencies detected.".into(),
                code: LeadErrCode::Ref,
            };
            topo.iter().for_each(|cell_ref| {
                if let Some(cell) = self.cells.get_mut(&cell_ref) {
                    cell.set_eval(Eval::Err(err.to_owned()));
                }
            });
            Err(topo)
        }
    }
}
