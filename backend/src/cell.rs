use std::collections::HashSet;

use crate::evaluator::*;

#[derive(Clone)]
pub struct Cell {
    eval: Eval,
    raw: String,
    i_dep: HashSet<CellRef>,
    they_dep: HashSet<CellRef>,
}

impl Cell {
    pub fn new(eval: Eval, raw: String) -> Self {
        Self {
            eval,
            raw,
            i_dep: HashSet::new(),
            they_dep: HashSet::new(),
        }
    }

    pub fn raw(&self) -> String {
        self.raw.clone()
    }

    pub fn eval(&self) -> Eval {
        self.eval.clone()
    }

    pub fn add_i_dep(&mut self, dep: CellRef) {
        self.i_dep.insert(dep);
    }

    pub fn add_they_dep(&mut self, dep: CellRef) {
        self.they_dep.insert(dep);
    }

    pub fn clear_i_dep(&mut self) {
        self.i_dep.clear();
    }

    pub fn clear_they_dep(&mut self) {
        self.they_dep.clear();
    }

    pub fn set_eval(&mut self, eval: Eval) {
        self.eval = eval;
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CellRef {
    pub row: i64,
    pub col: i64,
}

impl CellRef {
    pub fn new(s: String) -> Result<CellRef, String> {
        let s = s.trim();
        let mut col: i64 = 0;
        let mut i = 0;

        // consume leading letters for the column
        for (idx, ch) in s.char_indices() {
            if ch.is_ascii_alphabetic() {
                let u = ch.to_ascii_uppercase() as u8;
                let val = (u - b'A' + 1) as i64; // A->1 ... Z->26
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

        if let Ok(row) = row_part.parse::<i64>() {
            Ok(CellRef { row, col })
        } else {
            Err(format!("Parse error: invalid row number."))
        }
    }
}
