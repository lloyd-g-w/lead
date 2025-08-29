#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Double(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String), // Could be a function
    Literal(Literal),
    Operator(char),
    Paren(char),
    Comma,
    Eof,
}

pub struct Tokenizer {
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new(input: &str) -> Result<Tokenizer, String> {
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
                tokens.push(Token::Identifier(ident));
            } else if c.is_ascii_digit() {
                // parse number
                let mut number = String::new();
                let mut is_decimal = false;

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        number.push(ch);
                        chars.next();
                    } else if ch == '.' && !is_decimal {
                        is_decimal = true;
                        number.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if is_decimal {
                    tokens.push(Token::Literal(Literal::Double(number.parse().unwrap())))
                } else {
                    tokens.push(Token::Literal(Literal::Integer(number.parse().unwrap())))
                };
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
            } else if "+-*/^!%&|".contains(c) {
                tokens.push(Token::Operator(c));
                chars.next();
            } else if "()".contains(c) {
                tokens.push(Token::Paren(c));
                chars.next();
            } else if c == ',' {
                tokens.push(Token::Comma);
                chars.next();
            } else {
                return Err(format!("Encountered unknown token char: {c}"));
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
    fn test_tokenizer() {
        let raw = "hello hello 1.23 this 5 (1+2)";
        let expected: Vec<Token> = vec![
            Token::Identifier("hello".to_string()),
            Token::Identifier("hello".to_string()),
            Token::Literal(Literal::Double(1.23)),
            Token::Identifier("this".to_string()),
            Token::Literal(Literal::Integer(5)),
            Token::Paren('('),
            Token::Literal(Literal::Integer(1)),
            Token::Operator('+'),
            Token::Literal(Literal::Integer(2)),
            Token::Paren(')'),
        ];
        let t = Tokenizer::new(&raw).unwrap();
        assert_eq!(t.tokens, expected);
    }
}
