use serde::{Deserialize, Serialize};

use crate::{
    cell::CellRef,
    common::{LeadErr, LeadErrCode, Literal},
    evaluator::utils::*,
    grid::Grid,
    parser::*,
};

use std::{collections::HashSet, f64, fmt};

mod utils;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Eval {
    Literal(Literal),
    CellRef { eval: Box<Eval>, reference: CellRef },
    Range(Vec<Eval>),
    Err(LeadErr),
    Unset,
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eval::Literal(lit) => write!(f, "{lit:?}"),
            Eval::Range(it) => write!(f, "Range({it:?})"),
            Eval::CellRef { eval, reference } => write!(f, "EvalRef({eval:?}, {reference:?})"),
            Eval::Unset => write!(f, "Unset"),
            Eval::Err(it) => write!(f, "{it:?}"),
        }
    }
}

pub fn evaluate(str: String, grid: Option<&Grid>) -> (Eval, HashSet<CellRef>) {
    match parse(&str) {
        Ok((expr, _)) => {
            let mut precs = HashSet::new();

            match evaluate_expr(&expr, &mut precs, grid) {
                Ok(it) => (it, precs),
                Err(it) => (Eval::Err(it), precs),
            }
        }
        Err(e) => (Eval::Err(e), HashSet::new()),
    }
}

fn evaluate_expr(
    expr: &Expr,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
) -> Result<Eval, LeadErr> {
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
                return Err(LeadErr {
                    title: "Evaluation error.".into(),
                    desc: "Found cell reference with no grid.".into(),
                    code: LeadErrCode::Server,
                });
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
                _ => {
                    return Err(LeadErr {
                        title: "Evaluation error.".into(),
                        desc: format!("Unsupported operator {:?}", op),
                        code: LeadErrCode::Unsupported,
                    });
                }
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
            "EXP" => eval_single_arg_numeric(args, precs, grid, |x| x.exp(), "EXP".into())?,
            "SIN" => eval_single_arg_numeric(args, precs, grid, |x| x.sin(), "SIN".into())?,
            "COS" => eval_single_arg_numeric(args, precs, grid, |x| x.cos(), "COS".into())?,
            "TAN" => eval_single_arg_numeric(args, precs, grid, |x| x.tan(), "TAN".into())?,
            "ASIN" => eval_single_arg_numeric(args, precs, grid, |x| x.asin(), "ASIN".into())?,
            "ACOS" => eval_single_arg_numeric(args, precs, grid, |x| x.acos(), "ACOS".into())?,
            "ATAN" => eval_single_arg_numeric(args, precs, grid, |x| x.atan(), "ATAN".into())?,
            "PI" => eval_const(args, Eval::Literal(Literal::Number(f64::consts::PI)))?,
            "TAU" => eval_const(args, Eval::Literal(Literal::Number(f64::consts::TAU)))?,
            it => {
                return Err(LeadErr {
                    title: "Evaluation error.".into(),
                    desc: format!("Unsupported function {:?}", it),
                    code: LeadErrCode::Unsupported,
                });
            }
        },
        it => {
            return Err(LeadErr {
                title: "Evaluation error.".into(),
                desc: format!("Unsupported expression {:?}", it),
                code: LeadErrCode::Unsupported,
            });
        }
    };

    Ok(res)
}

fn eval_range(
    lval: &Eval,
    rval: &Eval,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
) -> Result<Eval, LeadErr> {
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
                        return Err(LeadErr {
                            title: "Evaluation error.".into(),
                            desc: "Found cell range but no grid.".into(),
                            code: LeadErrCode::Server,
                        });
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
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected cell reference types for RANGE function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_avg(
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
) -> Result<Eval, LeadErr> {
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
                        return Err(LeadErr {
                            title: "Evaluation error.".into(),
                            desc: "Found non-cellref in RANGE during AVG evaluation.".into(),
                            code: LeadErrCode::Server,
                        });
                    };

                    if let Eval::Literal(Literal::Number(num)) = *eval {
                        res += num;
                        count += 1;
                    } else if matches!(*eval, Eval::Unset) {
                        continue;
                    } else {
                        return Err(LeadErr {
                            title: "Evaluation error.".into(),
                            desc: "Expected numeric types for AVG function.".into(),
                            code: LeadErrCode::Unsupported,
                        });
                    }
                }
            }
            _ => {
                return Err(LeadErr {
                    title: "Evaluation error.".into(),
                    desc: "Expected numeric types for AVG function.".into(),
                    code: LeadErrCode::Unsupported,
                });
            }
        }
    }

    if count == 0 {
        Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Attempted to divide by zero.".into(),
            code: LeadErrCode::DivZero,
        })
    } else {
        Ok(Eval::Literal(Literal::Number(res / count as f64)))
    }
}

fn eval_const(args: &Vec<Expr>, value: Eval) -> Result<Eval, LeadErr> {
    if args.len() != 0 {
        return Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: format!("PI function requires no arguments."),
            code: LeadErrCode::Invalid,
        });
    }

    Ok(value)
}

fn eval_add(lval: &Eval, rval: &Eval) -> Result<Eval, LeadErr> {
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

            Err(LeadErr {
                title: "Evaluation error.".into(),
                desc: "Expected string or numeric types for ADD function.".into(),
                code: LeadErrCode::Unsupported,
            })
        }
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected string or numeric types for ADD function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_sub(lval: &Eval, rval: &Eval) -> Result<Eval, LeadErr> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x - y) {
                return Ok(Eval::Literal(res));
            }

            Err(LeadErr {
                title: "Evaluation error.".into(),
                desc: "Expected numeric types for SUB function.".into(),
                code: LeadErrCode::Unsupported,
            })
        }
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected numeric types for SUB function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_mul(lval: &Eval, rval: &Eval) -> Result<Eval, LeadErr> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Some(res) = eval_numeric_infix(a, b, |x, y| x * y) {
                return Ok(Eval::Literal(res));
            }

            Err(LeadErr {
                title: "Evaluation error.".into(),
                desc: "Expected numeric types for MUL function.".into(),
                code: LeadErrCode::Unsupported,
            })
        }
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected numeric types for MUL function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_div(lval: &Eval, rval: &Eval) -> Result<Eval, LeadErr> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let (Literal::Number(_), Literal::Number(y)) = (a, b) {
                if *y == 0f64 {
                    return Err(LeadErr {
                        title: "Evaluation error.".into(),
                        desc: "Attempted to divide by zero.".into(),
                        code: LeadErrCode::DivZero,
                    });
                }
            }

            if let Some(res) = eval_numeric_infix(a, b, |x, y| x / y) {
                return Ok(Eval::Literal(res));
            }

            Err(LeadErr {
                title: "Evaluation error.".into(),
                desc: "Expected numeric types for DIV function.".into(),
                code: LeadErrCode::Unsupported,
            })
        }
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected numeric types for DIV function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_pos(val: &Eval) -> Result<Eval, LeadErr> {
    match val {
        Eval::Literal(Literal::Number(it)) => Ok(Eval::Literal(Literal::Number(*it))),
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected numeric type for POS function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_neg(val: &Eval) -> Result<Eval, LeadErr> {
    match val {
        Eval::Literal(Literal::Number(it)) => Ok(Eval::Literal(Literal::Number(-it))),
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected numeric type for NEG function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_not(val: &Eval) -> Result<Eval, LeadErr> {
    match val {
        Eval::Literal(Literal::Boolean(it)) => Ok(Eval::Literal(Literal::Boolean(!it))),
        _ => Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: "Expected boolean type for NOT function.".into(),
            code: LeadErrCode::Unsupported,
        }),
    }
}

fn eval_numeric_infix(lhs: &Literal, rhs: &Literal, op: fn(f64, f64) -> f64) -> Option<Literal> {
    match (lhs, rhs) {
        (Literal::Number(a), Literal::Number(b)) => Some(Literal::Number(op(*a, *b))),
        _ => None,
    }
}
