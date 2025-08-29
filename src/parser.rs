use crate::tokenizer::*;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PrefixOp {
    POS,
    NEG,
    NOT,
}

#[derive(Debug, PartialEq)]
pub enum PostfixOp {
    PERCENT,
}

#[derive(Debug, PartialEq)]
pub enum InfixOp {
    MUL,
    DIV,
    ADD,
    SUB,
    AND,
    OR,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    CellRef(String),
    Function {
        name: String,
        args: Vec<Expr>,
    },
    Group(Box<Expr>),
    Prefix {
        op: PrefixOp,
        expr: Box<Expr>,
    },
    Postfix {
        op: PostfixOp,
        expr: Box<Expr>,
    },
    Infix {
        op: InfixOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

// Ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
// We have left and right precedence as to allow associative operators
// to parse as you would more expect and to break ties in a predictable manner
pub trait Precedence {
    fn prec(&self) -> (u8, u8);
}

impl Precedence for InfixOp {
    fn prec(&self) -> (u8, u8) {
        match self {
            InfixOp::MUL | InfixOp::DIV | InfixOp::AND => (3, 4),
            InfixOp::ADD | InfixOp::SUB | InfixOp::OR => (1, 2),
        }
    }
}
impl Precedence for PrefixOp {
    fn prec(&self) -> (u8, u8) {
        match self {
            _it => (0, 5),
        }
    }
}
impl Precedence for PostfixOp {
    fn prec(&self) -> (u8, u8) {
        match self {
            _it => (6, 0),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{lit:?}"),
            Expr::Group(expr) => write!(f, "({expr})"),
            Expr::Prefix { op, expr } => write!(f, "({op:?} {expr})"),
            Expr::Postfix { op, expr } => write!(f, "({op:?} {expr})"),
            Expr::Infix { op, lhs, rhs } => write!(f, "({lhs} {op:?} {rhs})"),
            Expr::Function { name, args } => write!(f, "{name}({args:?})"),
            Expr::CellRef(it) => write!(f, "CellRef({it})"),
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
            Expr::CellRef(_) => {}
            Expr::Group(expr) => {
                result.push_str(&expr.pretty_branch(&new_prefix, true));
            }
            Expr::Prefix { expr, .. } => {
                result.push_str(&expr.pretty_branch(&new_prefix, true));
            }
            Expr::Postfix { expr, .. } => {
                result.push_str(&expr.pretty_branch(&new_prefix, true));
            }
            Expr::Infix { lhs, rhs, .. } => {
                result.push_str(&lhs.pretty_branch(&new_prefix, false));
                result.push_str(&rhs.pretty_branch(&new_prefix, true));
            }
            Expr::Function { args, .. } => {
                for (idx, arg) in args.iter().enumerate() {
                    result.push_str(&arg.pretty_branch(&new_prefix, idx == args.len() - 1));
                }
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
            Expr::Postfix { op, .. } => format!("Postfix({:?})", op),
            Expr::Infix { op, .. } => format!("Infix({:?})", op),
            Expr::Function { name, .. } => format!("Function({:?})", name),
            Expr::CellRef(it) => format!("CellRef({:?})", it),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let mut tokenizer = Tokenizer::new(input)?;
    // println!("{:?}", tokenizer.tokens);
    _parse(&mut tokenizer, 0)
}

pub fn _parse(input: &mut Tokenizer, min_prec: u8) -> Result<Expr, String> {
    let mut lhs = match input.next() {
        Token::Literal(it) => Expr::Literal(it),
        Token::Identifier(id) if id == "true" => Expr::Literal(Literal::Boolean(true)),
        Token::Identifier(id) if id == "false" => Expr::Literal(Literal::Boolean(false)),
        Token::Paren('(') => {
            let lhs = _parse(input, 0)?;
            if input.next() != Token::Paren(')') {
                return Err(format!("Parse error: expected closing paren."));
            }
            Expr::Group(Box::new(lhs))
        }
        Token::Operator(op) => {
            let prefix_op = match op {
                '+' => PrefixOp::POS,
                '-' => PrefixOp::NEG,
                '!' => PrefixOp::NOT,
                it => return Err(format!("Parse error: unknown prefix operator {:?}.", it)),
            };

            let rhs = _parse(input, prefix_op.prec().1)?;

            Expr::Prefix {
                op: prefix_op,
                expr: Box::new(rhs),
            }
        }
        Token::Identifier(id) => match input.peek() {
            Token::Paren('(') => {
                input.next();

                let mut args: Vec<Expr> = Vec::new();
                loop {
                    let nxt = input.peek();

                    if nxt == Token::Paren(')') {
                        input.next();
                        break;
                    } else if nxt != Token::Comma && args.len() != 0 {
                        return Err(format!(
                            "Parse error: expected comma while parsing argument of function {:?}.",
                            id
                        ));
                    }

                    if args.len() != 0 {
                        input.next(); // Skip comma
                    }

                    let arg = _parse(input, 0)?;
                    args.push(arg);
                }

                Expr::Function {
                    name: id,
                    args: args,
                }
            }
            _ => Expr::CellRef(id),
        },

        it => return Err(format!("Parse error: did not expect token {:?}.", it)),
    };

    // In the reference article this is a loop with match
    // statement that breaks on Eof and closing paren but this is simpler and works as expected
    while let Token::Operator(op) = input.peek() {
        if "+-*/&|".contains(op) {
            let infix_op = match op {
                '+' => InfixOp::ADD,
                '-' => InfixOp::SUB,
                '*' => InfixOp::MUL,
                '/' => InfixOp::DIV,
                '&' => InfixOp::AND,
                '|' => InfixOp::OR,
                it => {
                    return Err(format!("Parse error: do not know infix operator {:?}.", it));
                }
            };

            let (l_prec, r_prec) = infix_op.prec();
            if l_prec < min_prec {
                break;
            }

            input.next();
            let rhs = _parse(input, r_prec)?;
            lhs = Expr::Infix {
                op: infix_op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        } else if "%".contains(op) {
            let postfix_op = match op {
                '%' => PostfixOp::PERCENT,
                it => {
                    return Err(format!(
                        "Parse error: do not know postfix operator {:?}.",
                        it
                    ));
                }
            };

            let (l_prec, _) = postfix_op.prec();
            if l_prec < min_prec {
                break;
            }

            input.next();
            lhs = Expr::Postfix {
                op: postfix_op,
                expr: Box::new(lhs),
            };
        }
    }

    Ok(lhs)
}
