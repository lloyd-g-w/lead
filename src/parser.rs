use crate::tokenizer::*;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PrefixOp {}

pub trait Precedence {
    fn prec(&self) -> u8;
}

#[derive(Debug, PartialEq)]
pub enum InfixOp {
    MUL,
    DIV,
    ADD,
    SUB,
}

impl Precedence for InfixOp {
    fn prec(&self) -> u8 {
        match self {
            InfixOp::MUL | InfixOp::DIV => 2,
            InfixOp::ADD | InfixOp::SUB => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Group(Box<Expr>),
    Prefix {
        op: PrefixOp,
        expr: Box<Expr>,
    },
    Infix {
        op: InfixOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{lit:?}"),
            Expr::Group(expr) => write!(f, "({expr})"),
            Expr::Prefix { op, expr } => write!(f, "({op:?} {expr})"),
            Expr::Infix { op, lhs, rhs } => write!(f, "({lhs} {op:?} {rhs})"),
        }
    }
}

impl Expr {
    pub fn pretty(&self) -> String {
        // entry point for users — root printed without └──
        let mut result = String::new();
        result.push_str(&format!("{}\n", self.node_name()));
        result.push_str(&self.pretty_subtree("", true));
        result
    }

    fn pretty_subtree(&self, prefix: &str, is_tail: bool) -> String {
        let mut result = String::new();
        let new_prefix = if is_tail {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        match self {
            Expr::Literal(_) => {}
            Expr::Group(expr) => {
                result.push_str(&expr.pretty_branch(&new_prefix, true));
            }
            Expr::Prefix { expr, .. } => {
                result.push_str(&expr.pretty_branch(&new_prefix, true));
            }
            Expr::Infix { lhs, rhs, .. } => {
                result.push_str(&lhs.pretty_branch(&new_prefix, false));
                result.push_str(&rhs.pretty_branch(&new_prefix, true));
            }
        }
        result
    }

    fn pretty_branch(&self, prefix: &str, is_tail: bool) -> String {
        let mut result = String::new();
        let branch = if is_tail { "└── " } else { "├── " };
        result.push_str(&format!("{}{}{}\n", prefix, branch, self.node_name()));
        result.push_str(&self.pretty_subtree(prefix, is_tail));
        result
    }

    fn node_name(&self) -> String {
        match self {
            Expr::Literal(lit) => format!("Literal({:?})", lit),
            Expr::Group(_) => "Group".to_string(),
            Expr::Prefix { op, .. } => format!("Prefix({:?})", op),
            Expr::Infix { op, .. } => format!("Infix({:?})", op),
        }
    }
}

pub fn parse(input: &mut Tokenizer) -> Result<Expr, String> {
    _parse(input, 0)
}

pub fn _parse(input: &mut Tokenizer, min_prec: u8) -> Result<Expr, String> {
    let mut lhs = match input.next() {
        Token::Literal(it) => Expr::Literal(it),
        it => return Err(format!("Parse error: did not expect token {:?}.", it)),
    };

    loop {
        let op = match input.peek() {
            Token::Eof => break,
            Token::Operator(op) => match op {
                '+' => InfixOp::ADD,
                '-' => InfixOp::SUB,
                '*' => InfixOp::MUL,
                '/' => InfixOp::DIV,
                it => return Err(format!("Parse error: do not know operator {:?}.", it)),
            },
            it => return Err(format!("Parse error: did not expect token {:?}.", it)),
        };

        if op.prec() < min_prec {
            break;
        }

        input.next();
        lhs = {
            let rhs = _parse(input, op.prec()).unwrap();
            Expr::Infix {
                op: op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
    }

    Ok(lhs)
}
