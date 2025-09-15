use std::collections::HashSet;

use crate::{
    cell::CellRef,
    common::{LeadErr, LeadErrCode, Literal},
    evaluator::{Eval, evaluate_expr},
    grid::Grid,
    parser::Expr,
};

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

macro_rules! const_func {
    ($fn_name:ident, $value:expr, $label:expr) => {
        pub fn $fn_name(args: &Vec<Expr>) -> Result<Eval, LeadErr> {
            eval_const(args, $value, $label)
        }
    };
}

const_func!(
    eval_pi,
    Eval::Literal(Literal::Number(std::f64::consts::PI)),
    "PI"
);

const_func!(
    eval_tau,
    Eval::Literal(Literal::Number(std::f64::consts::TAU)),
    "TAU"
);

const_func!(
    eval_sqrt2,
    Eval::Literal(Literal::Number(std::f64::consts::SQRT_2)),
    "SQRT2"
);

// -------------------------------------------------- //

fn eval_unary(
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

macro_rules! unary_func {
    ($fn_name:ident, $func:expr, $label:expr) => {
        pub fn $fn_name(
            args: &Vec<Expr>,
            precs: &mut HashSet<CellRef>,
            grid: Option<&Grid>,
        ) -> Result<Eval, LeadErr> {
            eval_unary(args, precs, grid, $func, $label)
        }
    };
}

unary_func!(eval_exp, |x| x.exp(), "EXP");
unary_func!(eval_log, |x| x.ln(), "LOG");
unary_func!(eval_sqrt, |x| x.sqrt(), "SQRT");
unary_func!(eval_abs, |x| x.abs(), "ABS");

unary_func!(eval_sin, |x| x.sin(), "SIN");
unary_func!(eval_cos, |x| x.cos(), "COS");
unary_func!(eval_tan, |x| x.tan(), "TAN");

unary_func!(eval_asin, |x| x.asin(), "ASIN");
unary_func!(eval_acos, |x| x.acos(), "ACOS");
unary_func!(eval_atan, |x| x.atan(), "ATAN");

// -------------------------------------------------- //

fn eval_infix(
    lhs: &Eval,
    rhs: &Eval,
    func: fn(f64, f64) -> f64,
    func_name: &str,
) -> Result<Eval, LeadErr> {
    let err = LeadErr {
        title: "Evaluation error.".into(),
        desc: format!("{func_name} function requires a numeric argument."),
        code: LeadErrCode::TypeErr,
    };

    let l = match lhs.to_owned() {
        Eval::Literal(Literal::Number(num)) => num,
        Eval::CellRef { eval, .. } => match *eval {
            Eval::Literal(Literal::Number(num)) => num,
            _ => return Err(err),
        },
        _ => return Err(err),
    };

    let r = match rhs.to_owned() {
        Eval::Literal(Literal::Number(num)) => num,
        Eval::CellRef { eval, .. } => match *eval {
            Eval::Literal(Literal::Number(num)) => num,
            _ => return Err(err),
        },
        _ => return Err(err),
    };

    Ok(Eval::Literal(Literal::Number(func(l, r))))
}

macro_rules! infix {
    ($fn_name:ident, $func:expr, $label:expr) => {
        pub fn $fn_name(lhs: &Eval, rhs: &Eval) -> Result<Eval, LeadErr> {
            eval_infix(lhs, rhs, $func, $label)
        }
    };
}

// Can concat string as well
pub fn eval_add(lval: &Eval, rval: &Eval) -> Result<Eval, LeadErr> {
    match (lval, rval) {
        (Eval::Literal(a), Eval::Literal(b)) => {
            if let Ok(res) = eval_infix(lval, rval, |x, y| x + y, "ADD") {
                return Ok(res);
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

infix!(eval_sub, |x, y| x - y, "SUB");
infix!(eval_mul, |x, y| x * y, "MUL");
infix!(eval_div, |x, y| x / y, "DIV");
