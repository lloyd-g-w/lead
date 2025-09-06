use crate::cell::CellRef;
use crate::grid::Grid;
use crate::parser::*;
use crate::tokenizer::Literal;
use std::collections::HashSet;
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

pub fn evaluate(str: String, grid: Option<&Grid>) -> Result<(Eval, HashSet<CellRef>), String> {
    let (expr, deps) = parse(&str)?;

    match evaluate_expr(&expr, grid) {
        Ok(it) => Ok((it, deps)),
        Err(it) => Err(it),
    }
}

fn evaluate_expr(expr: &Expr, grid: Option<&Grid>) -> Result<Eval, String> {
    let res = match expr {
        Expr::Literal(lit) => Eval::Literal(lit.clone()),
        Expr::CellRef(re) => {
            if let Some(g) = grid {
                g.get_cell(re.to_owned())?
            } else {
                return Err("Evaluation error: Found cell reference but no grid.".into());
            }
        }
        Expr::Infix { op, lhs, rhs } => {
            let lval = evaluate_expr(lhs, grid)?;
            let rval = evaluate_expr(rhs, grid)?;

            match op {
                InfixOp::ADD => eval_add(&lval, &rval)?,
                InfixOp::SUB => eval_sub(&lval, &rval)?,
                InfixOp::MUL => eval_mul(&lval, &rval)?,
                InfixOp::DIV => eval_div(&lval, &rval)?,
                _ => return Err(format!("Evaluation error: Unsupported operator {:?}", op)),
            }
        }
        Expr::Prefix { op, expr } => {
            let val = evaluate_expr(expr, grid)?;

            match op {
                PrefixOp::POS => eval_pos(&val)?,
                PrefixOp::NEG => eval_neg(&val)?,
                PrefixOp::NOT => eval_not(&val)?,
                // _ => return Err(format!("Evaluation error: Unsupported operator {:?}", op)),
            }
        }
        Expr::Group(g) => evaluate_expr(g, grid)?,
        it => return Err(format!("Evaluation error: Unsupported expression {:?}", it)),
    };

    Ok(res)
}

fn eval_add(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x + y) {
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
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x - y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected string or numeric types for SUB function.".to_string())
        }
    }
}
fn eval_mul(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x * y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected string or numeric types for MUL function.".to_string())
        }
    }
}
fn eval_div(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let (Literal::Number(_), Literal::Number(y)) = (a, b) {
                if *y == 0f64 {
                    return Err(
                        "Evaluation error: integers attempted to divide by zero.".to_string()
                    );
                }
            }

            if let Some(res) = eval_numeric_infix(a, b, |x, y| x / y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected string or numeric types for DIV function.".to_string())
        }
    }
}

fn eval_numeric_infix(lhs: &Literal, rhs: &Literal, op: fn(f64, f64) -> f64) -> Option<Literal> {
    match (lhs, rhs) {
        (Literal::Number(a), Literal::Number(b)) => Some(Literal::Number(op(*a, *b))),
        _ => None,
    }
}

fn eval_pos(val: &Eval) -> Result<Eval, String> {
    match val {
        Eval::Literal(Literal::Number(it)) => Ok(Eval::Literal(Literal::Number(*it))),
        _ => Err("Evaluation error: expected numeric type for POS function.".to_string()),
    }
}

fn eval_neg(val: &Eval) -> Result<Eval, String> {
    match val {
        Eval::Literal(Literal::Number(it)) => Ok(Eval::Literal(Literal::Number(-it))),
        _ => Err("Evaluation error: expected numeric type for NEG function.".to_string()),
    }
}
fn eval_not(val: &Eval) -> Result<Eval, String> {
    match val {
        Eval::Literal(Literal::Boolean(it)) => Ok(Eval::Literal(Literal::Boolean(!it))),
        _ => Err("Evaluation error: expected boolean type for NEG function.".to_string()),
    }
}
