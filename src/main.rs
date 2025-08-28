mod parser;
mod tokenizer;

use crate::parser::*;
use crate::tokenizer::*;
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Expected input.");

    let mut t = Tokenizer::new(&input).unwrap();
    println!("{:?}", t.tokens);
    let ast = parser::parse(&mut t).unwrap();
    println!("{}", ast.pretty());
}
