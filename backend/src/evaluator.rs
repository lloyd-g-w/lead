use crate::cell::{Cell, CellRef};
use crate::parser::*;
use crate::tokenizer::Literal;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Eval {
    Literal(Literal),
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eval::Literal(lit) => write!(f, "{lit:?}"),
        }
    }
}

pub struct Evaluator {
    cells: HashMap<CellRef, Cell>,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        return Evaluator {
            cells: HashMap::new(),
        };
    }

    pub fn set_cell(&mut self, cell_ref: CellRef, raw_val: String) -> Result<(), String> {
        if self.cells.contains_key(&cell_ref) && self.cells[&cell_ref].raw() == raw_val {
            return Ok(());
        }

        let eval: Eval;

        if let Some(c) = raw_val.chars().nth(0)
            && c == '='
        {
            eval = self.evaluate(raw_val[1..].to_owned())?;
        } else {
            match self.evaluate(raw_val.to_owned()) {
                Ok(e) => {
                    eval = e;
                }
                Err(_) => eval = Eval::Literal(Literal::String(raw_val.to_owned())),
            }
        }

        self.cells.insert(cell_ref, Cell::new(eval, raw_val));
        Ok(())
    }

    pub fn get_cell(&mut self, cell_ref: CellRef) -> Result<(String, Eval), String> {
        if !self.cells.contains_key(&cell_ref) {
            return Err(format!("Cell at {:?} not found.", cell_ref));
        }

        let cell = &self.cells[&cell_ref];

        Ok((cell.raw(), cell.eval()))
    }

    pub fn evaluate(&mut self, str: String) -> Result<Eval, String> {
        let (mut expr, mut deps) = parse(&str)?;

        self.evaluate_expr(&mut expr)
    }

    fn evaluate_expr(&mut self, expr: &mut Expr) -> Result<Eval, String> {
        let res = match expr {
            Expr::Literal(lit) => Eval::Literal(lit.clone()),
            Expr::CellRef(re) => self.get_cell(re.to_owned())?.1,
            Expr::Infix { op, lhs, rhs } => {
                let lval = self.evaluate_expr(lhs)?;
                let rval = self.evaluate_expr(rhs)?;

                match op {
                    InfixOp::ADD => eval_add(&lval, &rval)?,
                    InfixOp::SUB => eval_sub(&lval, &rval)?,
                    InfixOp::MUL => eval_mul(&lval, &rval)?,
                    InfixOp::DIV => eval_div(&lval, &rval)?,
                    _ => return Err(format!("Evaluation error: Unsupported operator {:?}", op)),
                }
            }
            it => return Err(format!("Evaluation error: Unsupported expression {:?}", it)),
        };

        Ok(res)
    }
}

fn eval_add(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x + y, |x, y| x + y) {
                return Ok(Eval::Literal(res));
            }

            // Try string concatenation
            if let (Literal::String(x), Literal::String(y)) = (a, b) {
                let mut res = x.to_owned();
                res.push_str(y);
                return Ok(Eval::Literal(Literal::String(res)));
            }

            Err("Evaluation error: expected string or numeric types for ADD function.".to_string())
        }
    }
}

fn eval_sub(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x - y, |x, y| x - y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected string or numeric types for SUB function.".to_string())
        }
    }
}
fn eval_mul(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x * y, |x, y| x * y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected string or numeric types for MUL function.".to_string())
        }
    }
}
fn eval_div(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x / y, |x, y| x / y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected string or numeric types for DIV function.".to_string())
        }
    }
}

fn eval_numeric_infix<FInt, FDouble>(
    lhs: &Literal,
    rhs: &Literal,
    int_op: FInt,
    double_op: FDouble,
) -> Option<Literal>
where
    FInt: Fn(i64, i64) -> i64,
    FDouble: Fn(f64, f64) -> f64,
{
    match (lhs, rhs) {
        (Literal::Integer(a), Literal::Integer(b)) => Some(Literal::Integer(int_op(*a, *b))),
        (Literal::Double(a), Literal::Double(b)) => Some(Literal::Double(double_op(*a, *b))),
        (Literal::Integer(a), Literal::Double(b)) => {
            Some(Literal::Double(double_op(*a as f64, *b)))
        }
        (Literal::Double(a), Literal::Integer(b)) => {
            Some(Literal::Double(double_op(*a, *b as f64)))
        }
        _ => None,
    }
}
