use crate::common::{LeadErr, LeadErrCode, Literal};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String), // Could be a function
    Literal(Literal),
    Operator(char),
    Err(LeadErr),
    OpenParen,
    CloseParen,
    Comma,
    Eof,
}

pub const OPERATORS_STR: &str = "+-*/^!%&|:";

pub struct Tokenizer {
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new(input: &str) -> Result<Tokenizer, LeadErr> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else if c.is_ascii_alphabetic() {
                // parse identifier
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let res = match ident.as_str() {
                    "true" => Token::Literal(Literal::Boolean(true)),
                    "false" => Token::Literal(Literal::Boolean(false)),
                    it => Token::Identifier(it.into()),
                };

                tokens.push(res);
            } else if c.is_ascii_digit() {
                // parse number
                let mut number = String::new();
                let mut is_decimal = false;
                let mut is_exp = false;

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        number.push(ch);
                        chars.next();
                    } else if ch == '.' && !is_decimal && !is_exp {
                        is_decimal = true;
                        number.push(ch);
                        chars.next();
                    } else if ch == 'e' && !is_decimal && !is_exp {
                        is_exp = true;
                        number.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // TODO: REMOVE UNWRAP
                tokens.push(Token::Literal(Literal::Number(number.parse().unwrap())));
            } else if c == '"' || c == '\'' {
                // parse string literal
                let mut string = String::new();

                let quote = c;
                let mut escapes = 0;
                chars.next(); // consume opening quote

                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == quote && escapes % 2 == 0 {
                        break;
                    }

                    if ch == '\\' {
                        escapes += 1;
                    } else {
                        escapes = 0;
                    }

                    string.push(ch);
                }
                tokens.push(Token::Literal(Literal::String(string)));
            } else if OPERATORS_STR.contains(c) {
                tokens.push(Token::Operator(c));
                chars.next();
            } else if "()".contains(c) {
                if c == '(' {
                    tokens.push(Token::OpenParen);
                } else {
                    tokens.push(Token::CloseParen);
                }
                chars.next();
            } else if c == ',' {
                tokens.push(Token::Comma);
                chars.next();
            } else {
                return Err(LeadErr {
                    title: "Tokenizer error.".into(),
                    desc: format!("Encountered unknown token char: {c}"),
                    code: LeadErrCode::Syntax,
                });
            }
        }

        tokens.reverse(); // Since we want FIFO and next + peek are implemented as LILO
        Ok(Tokenizer { tokens })
    }

    pub fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }
    pub fn peek(&mut self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_token() {
        assert_eq!(
            Tokenizer::new("1").unwrap().tokens,
            Vec::from([Token::Literal(Literal::Number(1.0))])
        );
        assert_eq!(
            Tokenizer::new("2.0").unwrap().tokens,
            Vec::from([Token::Literal(Literal::Number(2.0))])
        );
        assert_eq!(
            Tokenizer::new("\"hello\"").unwrap().tokens,
            Vec::from([Token::Literal(Literal::String("hello".into()))])
        );
        assert_eq!(
            Tokenizer::new("\'hello\'").unwrap().tokens,
            Vec::from([Token::Literal(Literal::String("hello".into()))])
        );
        assert_eq!(
            Tokenizer::new("hello").unwrap().tokens,
            Vec::from([Token::Identifier("hello".into())])
        );
        assert_eq!(
            Tokenizer::new("+").unwrap().tokens,
            Vec::from([Token::Operator('+')])
        );
        assert_eq!(
            Tokenizer::new(",").unwrap().tokens,
            Vec::from([Token::Comma])
        );
        assert_eq!(
            Tokenizer::new(")").unwrap().tokens,
            Vec::from([Token::CloseParen])
        );
        assert_eq!(
            Tokenizer::new("(").unwrap().tokens,
            Vec::from([Token::OpenParen])
        );
    }

    #[test]
    fn test_token_punctuation() {
        let mut exp = Vec::from([
            Token::Comma,
            Token::CloseParen,
            Token::Comma,
            Token::OpenParen,
        ]);
        exp.reverse();

        assert_eq!(Tokenizer::new(", ) , (").unwrap().tokens, exp);
    }

    #[test]
    fn test_token_operators() {
        let mut exp = Vec::from([
            Token::Operator('+'),
            Token::Operator('-'),
            Token::Operator('*'),
            Token::Operator('/'),
            Token::Operator('^'),
            Token::Operator('!'),
            Token::Operator('%'),
            Token::Operator('&'),
            Token::Operator('|'),
        ]);
        exp.reverse();

        assert_eq!(Tokenizer::new("+-*/^!%&|").unwrap().tokens, exp);
    }

    #[test]
    fn test_token_string() {
        let raw = "\"hello\" \'world\'";
        let mut expected: Vec<Token> = vec![
            Token::Literal(Literal::String("hello".into())),
            Token::Literal(Literal::String("world".into())),
        ];
        expected.reverse();
        let t = Tokenizer::new(&raw).unwrap();
        assert_eq!(t.tokens, expected);
    }

    #[test]
    fn test_token_number() {
        let raw = "123 4.56";
        let mut expected: Vec<Token> = vec![
            Token::Literal(Literal::Number(123.0)),
            Token::Literal(Literal::Number(4.56)),
        ];
        expected.reverse();
        let t = Tokenizer::new(&raw).unwrap();
        assert_eq!(t.tokens, expected);
    }

    #[test]
    fn test_token_boolean() {
        let raw = "false true";
        let mut expected: Vec<Token> = vec![
            Token::Literal(Literal::Boolean(false)),
            Token::Literal(Literal::Boolean(true)),
        ];
        expected.reverse();
        let t = Tokenizer::new(&raw).unwrap();
        assert_eq!(t.tokens, expected);
    }

    #[test]
    fn test_token_identifier() {
        let raw = "hello test";
        let mut expected: Vec<Token> = vec![
            Token::Identifier("hello".to_string()),
            Token::Identifier("test".to_string()),
        ];
        expected.reverse();
        let t = Tokenizer::new(&raw).unwrap();
        assert_eq!(t.tokens, expected);
    }

    #[test]
    fn test_token_mix() {
        let raw = "hello test 1.23 this 5 (1+2)";
        let mut expected: Vec<Token> = vec![
            Token::Identifier("hello".to_string()),
            Token::Identifier("test".to_string()),
            Token::Literal(Literal::Number(1.23)),
            Token::Identifier("this".to_string()),
            Token::Literal(Literal::Number(5.0)),
            Token::OpenParen,
            Token::Literal(Literal::Number(1.0)),
            Token::Operator('+'),
            Token::Literal(Literal::Number(2.0)),
            Token::CloseParen,
        ];
        expected.reverse();
        let t = Tokenizer::new(&raw).unwrap();
        assert_eq!(t.tokens, expected);
    }
}
