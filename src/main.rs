mod evaluator;
mod parser;
mod tokenizer;

use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Expected input.");

    let mut ast = parser::parse(&input).unwrap();
    println!("{}", ast.pretty());
    println!("{}", evaluator::_evaluate(&mut ast).unwrap());
}
