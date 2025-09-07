use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::evaluator::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CellRef {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Debug)]
pub struct Cell {
    reference: CellRef,
    eval: Eval,
    raw: String,
    precedents: HashSet<CellRef>, // Cells that this cell reads
    dependents: HashSet<CellRef>, // Cells that read this cell
}

impl Cell {
    pub fn new(reference: CellRef, eval: Eval, raw: String) -> Self {
        Self {
            reference,
            eval,
            raw,
            precedents: HashSet::new(),
            dependents: HashSet::new(),
        }
    }

    pub fn new_all(
        reference: CellRef,
        eval: Eval,
        raw: String,
        precedents: HashSet<CellRef>,
        dependents: HashSet<CellRef>,
    ) -> Self {
        Self {
            reference,
            eval,
            raw,
            precedents,
            dependents,
        }
    }

    pub fn raw(&self) -> String {
        self.raw.to_owned()
    }
    pub fn eval(&self) -> Eval {
        self.eval.to_owned()
    }
    pub fn reference(&self) -> CellRef {
        self.reference.to_owned()
    }

    pub fn set_raw(&mut self, raw: String) {
        self.raw = raw;
    }
    pub fn set_eval(&mut self, eval: Eval) {
        self.eval = eval;
    }
    pub fn set_ref(&mut self, reference: CellRef) {
        self.reference = reference;
    }

    pub fn add_dep(&mut self, it: CellRef) {
        self.dependents.insert(it);
    }

    pub fn remove_dep(&mut self, it: &CellRef) {
        self.dependents.remove(&it);
    }

    pub fn add_prec(&mut self, it: CellRef) {
        self.precedents.insert(it);
    }

    pub fn set_precs(&mut self, it: HashSet<CellRef>) {
        self.precedents = it;
    }

    pub fn deps(&self) -> HashSet<CellRef> {
        self.dependents.to_owned()
    }

    pub fn precs(&self) -> HashSet<CellRef> {
        self.precedents.to_owned()
    }
}

impl CellRef {
    // Zero indexed
    pub fn new(s: String) -> Result<CellRef, String> {
        let s = s.trim();
        let mut col: usize = 0;
        let mut i = 0;

        // consume leading letters for the column
        for (idx, ch) in s.char_indices() {
            if ch.is_ascii_alphabetic() {
                let u = ch.to_ascii_uppercase() as u8;
                let val = (u - b'A' + 1) as usize; // A->1 ... Z->26
                col = col * 26 + val;
                i = idx + ch.len_utf8();
            } else {
                break;
            }
        }

        if col <= 0 {
            return Err(format!(
                "Parse error: missing column letters in cell ref: {s}"
            ));
        }

        let row_part = &s[i..];
        if row_part.is_empty() {
            return Err(format!(
                "Parse error: missing column letters in cell ref: {s}"
            ));
        } else if !row_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!(
                "Parse error: row part must be numeric in cell ref: {s}"
            ));
        }

        if let Ok(row) = row_part.parse::<usize>() {
            Ok(CellRef {
                row: row - 1,
                col: col - 1,
            })
        } else {
            Err(format!("Parse error: invalid row number."))
        }
    }
}
