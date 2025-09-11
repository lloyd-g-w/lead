use std::collections::HashSet;

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
