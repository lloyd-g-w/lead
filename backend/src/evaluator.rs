use serde::{Deserialize, Serialize};

use crate::cell::CellRef;
use crate::grid::Grid;
use crate::parser::*;
use crate::tokenizer::Literal;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Eval {
    Literal(Literal),
    CellRef { eval: Box<Eval>, reference: CellRef },
    Range(Vec<Eval>),
    Unset,
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eval::Literal(lit) => write!(f, "{lit:?}"),
            Eval::Range(it) => write!(f, "Range({it:?})"),
            Eval::CellRef { eval, reference } => write!(f, "EvalRef({eval:?}, {reference:?})"),
            Eval::Unset => write!(f, "Unset"),
        }
    }
}

pub fn evaluate(str: String, grid: Option<&Grid>) -> Result<(Eval, HashSet<CellRef>), String> {
    let (expr, _) = parse(&str)?;

    let mut precs = HashSet::new();

    // Make evaulator adds precs for ranges
    match evaluate_expr(&expr, &mut precs, grid) {
        Ok(it) => Ok((it, precs)),
        Err(it) => Err(it),
    }
}

fn evaluate_expr(
    expr: &Expr,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
) -> Result<Eval, String> {
    let res = match expr {
        Expr::Literal(lit) => Eval::Literal(lit.clone()),
        Expr::CellRef(re) => {
            if let Some(g) = grid {
                Eval::CellRef {
                    eval: Box::new(
                        g.get_cell(re.to_owned())
                            .map_or(Eval::Unset, |cell| cell.eval()),
                    ),
                    reference: {
                        precs.insert(*re);
                        *re
                    },
                }
            } else {
                return Err("Evaluation error: Found cell reference but no grid.".into());
            }
        }
        Expr::Infix { op, lhs, rhs } => {
            let mut lval = evaluate_expr(lhs, precs, grid)?;
            let mut rval = evaluate_expr(rhs, precs, grid)?;

            if !matches!(op, InfixOp::RANGE) {
                if let Eval::CellRef { eval, reference: _ } = lval {
                    lval = *eval;
                }
                if let Eval::CellRef { eval, reference: _ } = rval {
                    rval = *eval;
                }
            }

            match op {
                InfixOp::ADD => eval_add(&lval, &rval)?,
                InfixOp::SUB => eval_sub(&lval, &rval)?,
                InfixOp::MUL => eval_mul(&lval, &rval)?,
                InfixOp::DIV => eval_div(&lval, &rval)?,
                InfixOp::RANGE => eval_range(&lval, &rval, precs, grid)?,
                _ => return Err(format!("Evaluation error: Unsupported operator {:?}", op)),
            }
        }
        Expr::Prefix { op, expr } => {
            let mut val = evaluate_expr(expr, precs, grid)?;

            if let Eval::CellRef { eval, reference: _ } = val {
                val = *eval;
            }

            match op {
                PrefixOp::POS => eval_pos(&val)?,
                PrefixOp::NEG => eval_neg(&val)?,
                PrefixOp::NOT => eval_not(&val)?,
                // _ => return Err(format!("Evaluation error: Unsupported operator {:?}", op)),
            }
        }
        Expr::Group(g) => evaluate_expr(g, precs, grid)?,
        Expr::Function { name, args } => match name.as_str() {
            "AVG" => eval_avg(args, precs, grid)?,
            it => return Err(format!("Evaluation error: Unsupported function {}.", it)),
        },
        it => return Err(format!("Evaluation error: Unsupported expression {:?}", it)),
    };

    Ok(res)
}

fn eval_range(
    lval: &Eval,
    rval: &Eval,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
) -> Result<Eval, String> {
    match (lval, rval) {
        (
            Eval::CellRef {
                eval: _,
                reference: a_ref,
            },
            Eval::CellRef {
                eval: _,
                reference: b_ref,
            },
        ) => {
            let mut cells = Vec::new();

            // assume row-major expansion
            let row_start = a_ref.row.min(b_ref.row);
            let row_end = a_ref.row.max(b_ref.row);
            let col_start = a_ref.col.min(b_ref.col);
            let col_end = a_ref.col.max(b_ref.col);

            for row in row_start..=row_end {
                for col in col_start..=col_end {
                    let reference = CellRef { row, col };

                    let Some(g) = grid else {
                        return Err("Evaluation error: Found cell range but no grid.".into());
                    };

                    cells.push(Eval::CellRef {
                        eval: Box::new(
                            g.get_cell(reference.to_owned())
                                .map_or(Eval::Unset, |cell| cell.eval()),
                        ),
                        reference: {
                            precs.insert(reference);
                            reference
                        },
                    });
                }
            }

            Ok(Eval::Range(cells))
        }
        _ => Err("Evaluation error: expected cellref types for RANGE function.".to_string()),
    }
}

fn eval_avg(
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
) -> Result<Eval, String> {
    let mut res = 0.0;
    let mut count = 0;

    for arg in args {
        match evaluate_expr(arg, precs, grid)? {
            Eval::Literal(Literal::Number(num)) => {
                res += num;
                count += 1;
            }
            Eval::Range(range) => {
                for cell in range {
                    let Eval::CellRef { eval, reference: _ } = cell else {
                        panic!("Found non cellref in evaluation time RANGE!.");
                    };

                    if let Eval::Literal(Literal::Number(num)) = *eval {
                        res += num;
                        count += 1;
                    } else if matches!(*eval, Eval::Unset) {
                        continue;
                    } else {
                        return Err("Evaluation error: expected numeric types for AVG function."
                            .to_string());
                    }
                }
            }
            _ => {
                return Err(
                    "Evaluation error: expected numeric types for AVG function.".to_string()
                );
            }
        }
    }

    if count == 0 {
        Err("Evaluation error: attempted to divide by zero.".to_string())
    } else {
        Ok(Eval::Literal(Literal::Number(res / count as f64)))
    }
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
        _ => {
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

            Err("Evaluation error: expected numeric types for SUB function.".to_string())
        }
        _ => Err("Evaluation error: expected numeric types for SUB function.".to_string()),
    }
}
fn eval_mul(lval: &Eval, rval: &Eval) -> Result<Eval, String> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x * y) {
                return Ok(Eval::Literal(res));
            }

            Err("Evaluation error: expected numeric types for MUL function.".to_string())
        }
        _ => Err("Evaluation error: expected numeric types for MUL function.".to_string()),
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

            Err("Evaluation error: expected numeric types for DIV function.".to_string())
        }
        _ => Err("Evaluation error: expected numeric types for DIV  function.".to_string()),
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
