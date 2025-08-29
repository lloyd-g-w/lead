use crate::parser::*;
use crate::tokenizer::Literal;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Eval {
    Literal(Literal),
    Expr(Expr),
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eval::Literal(lit) => write!(f, "{lit:?}"),
            Eval::Expr(expr) => write!(f, "({expr})"),
        }
    }
}

pub fn _evaluate(expr: &mut Expr) -> Result<Eval, String> {
    let res = match expr {
        Expr::Literal(lit) => Eval::Literal(lit.clone()),
        Expr::Infix { op, lhs, rhs } => {
            let lval = _evaluate(lhs)?;
            let rval = _evaluate(rhs)?;

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
        _ => return Err("Evalutation error: expected literals for ADD function.".to_string()),
    }
}

fn eval_sub(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    Err("Todo.".to_string())
}
fn eval_mul(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    Err("Todo.".to_string())
}
fn eval_div(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    Err("Todo.".to_string())
}

pub fn eval_numeric_infix<FInt, FDouble>(
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
