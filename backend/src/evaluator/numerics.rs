use std::collections::HashSet;

use crate::{
    cell::CellRef,
    common::{LeadErr, LeadErrCode, Literal},
    evaluator::{Eval, evaluate_expr},
    grid::Grid,
    parser::Expr,
};

// -------------------------------------------------- //

fn eval_unary_numeric(
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
    func: fn(f64) -> f64,
    func_name: &str,
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

macro_rules! unary_numeric_func {
    ($fn_name:ident, $func:expr, $label:expr) => {
        pub fn $fn_name(
            args: &Vec<Expr>,
            precs: &mut HashSet<CellRef>,
            grid: Option<&Grid>,
        ) -> Result<Eval, LeadErr> {
            eval_unary_numeric(args, precs, grid, $func, $label)
        }
    };
}

unary_numeric_func!(eval_exp, |x| x.exp(), "EXP");
unary_numeric_func!(eval_log, |x| x.ln(), "LOG");
unary_numeric_func!(eval_sqrt, |x| x.sqrt(), "SQRT");
unary_numeric_func!(eval_abs, |x| x.abs(), "ABS");

unary_numeric_func!(eval_sin, |x| x.sin(), "SIN");
unary_numeric_func!(eval_cos, |x| x.cos(), "COS");
unary_numeric_func!(eval_tan, |x| x.tan(), "TAN");

unary_numeric_func!(eval_asin, |x| x.asin(), "ASIN");
unary_numeric_func!(eval_acos, |x| x.acos(), "ACOS");
unary_numeric_func!(eval_atan, |x| x.atan(), "ATAN");

// -------------------------------------------------- //

fn eval_const(args: &Vec<Expr>, value: Eval, label: &str) -> Result<Eval, LeadErr> {
    if args.len() != 0 {
        return Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: format!("{label} function requires no arguments."),
            code: LeadErrCode::Invalid,
        });
    }

    Ok(value)
}

macro_rules! const_numeric_func {
    ($fn_name:ident, $value:expr, $label:expr) => {
        pub fn $fn_name(args: &Vec<Expr>) -> Result<Eval, LeadErr> {
            eval_const(args, $value, $label)
        }
    };
}

const_numeric_func!(
    eval_pi,
    Eval::Literal(Literal::Number(std::f64::consts::PI)),
    "PI"
);

const_numeric_func!(
    eval_tau,
    Eval::Literal(Literal::Number(std::f64::consts::TAU)),
    "TAU"
);

const_numeric_func!(
    eval_sqrt2,
    Eval::Literal(Literal::Number(std::f64::consts::SQRT_2)),
    "SQRT2"
);

// -------------------------------------------------- //
