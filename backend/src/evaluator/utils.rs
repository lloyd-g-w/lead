use std::{collections::HashSet, default};

use crate::{
    cell::CellRef,
    common::{LeadErr, LeadErrCode, Literal},
    evaluator::{Eval, evaluate_expr},
    grid::Grid,
    parser::Expr,
};

pub fn eval_single_arg_numeric(
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
    func: fn(f64) -> f64,
    func_name: String,
) -> Result<Eval, LeadErr> {
    if args.len() != 1 {
        return Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: format!("{func_name} function requires a single argument."),
            code: LeadErrCode::Invalid,
        });
    }
    let err = LeadErr {
        title: "Evaluation error.".into(),
        desc: format!("{func_name} function requires a numeric argument."),
        code: LeadErrCode::TypeErr,
    };
    match evaluate_expr(&args[0], precs, grid)? {
        Eval::Literal(Literal::Number(num)) => Ok(Eval::Literal(Literal::Number(func(num)))),
        Eval::CellRef { eval, .. } => match *eval {
            Eval::Literal(Literal::Number(n)) => Ok(Eval::Literal(Literal::Number(func(n)))),
            _ => Err(err),
        },
        _ => Err(err),
    }
}

pub fn eval_n_arg_numeric(
    n: usize,
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
    func: fn(Vec<f64>) -> f64,
    func_name: String,
) -> Result<Eval, LeadErr> {
    if args.len() != n {
        return Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: format!("{func_name} function requires {n} argument(s)."),
            code: LeadErrCode::Invalid,
        });
    }

    let err = LeadErr {
        title: "Evaluation error.".into(),
        desc: format!("{func_name} function requires numeric argument(s)."),
        code: LeadErrCode::TypeErr,
    };

    let mut numbers = Vec::with_capacity(n);

    for arg in args {
        match evaluate_expr(arg, precs, grid)? {
            Eval::Literal(Literal::Number(num)) => numbers.push(num),
            Eval::CellRef { eval, .. } => match *eval {
                Eval::Literal(Literal::Number(num)) => numbers.push(num),
                _ => return Err(err.clone()),
            },
            _ => return Err(err.clone()),
        }
    }

    Ok(Eval::Literal(Literal::Number(func(numbers))))
}

// This is a utility function that filters out and error handles all non literal numbers or unset
// eval types and handles ranges
pub fn eval_numeric_func(
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
    func: fn(Vec<Eval>) -> Result<f64, LeadErr>,
    func_name: String,
) -> Result<Eval, LeadErr> {
    let mut numeric_args = Vec::new();

    for arg in args {
        let eval = evaluate_expr(arg, precs, grid)?;

        if matches!(eval, Eval::Literal(Literal::Number(_)) | Eval::Unset) {
            numeric_args.push(eval);
        } else if matches!(eval, Eval::Range(_)) {
            if let Eval::Range(range) = eval {
                for cell in range {
                    let Eval::CellRef {
                        eval: eval2,
                        reference: _,
                    } = cell
                    else {
                        return Err(LeadErr {
                            title: "Evaluation error.".into(),
                            desc: format!(
                                "Found non-cellref in RANGE during {func_name} evaluation."
                            ),
                            code: LeadErrCode::Server,
                        });
                    };

                    if matches!(*eval2, Eval::Literal(Literal::Number(_)) | Eval::Unset) {
                        numeric_args.push(*eval2);
                    } else {
                        return Err(LeadErr {
                            title: "Evaluation error.".into(),
                            desc: format!("Expected numeric types for {func_name} function."),
                            code: LeadErrCode::Unsupported,
                        });
                    }
                }
            }
        } else {
            return Err(LeadErr {
                title: "Evaluation error.".into(),
                desc: format!("Expected numeric types for {func_name} function."),
                code: LeadErrCode::Unsupported,
            });
        }
    }

    let res = func(numeric_args)?;
    Ok(Eval::Literal(Literal::Number(res)))
}
