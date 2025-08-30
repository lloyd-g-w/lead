mod cell;
mod evaluator;
mod parser;
mod tokenizer;

use std::io;

use crate::{cell::CellRef, evaluator::Evaluator};

fn main() {
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).expect("Expected input.");

    // let mut ast = parser::parse(&input).unwrap();
    // println!("{}", ast.pretty());
    let mut evaluator = Evaluator::new();
    // // println!("{}", evaluator.evaluate(input).unwrap());
    // let a1 = CellRef { row: 1, col: 2 };
    // evaluator.set_cell(a1, input).unwrap();
    // println!("{:?}", evaluator.get_cell(a1).unwrap());

    println!("CMDS : set <cell_ref>, get <cell_ref>");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Expected input.");

        let cmds = ["set", "get"];
        let cmd = &input[0..3];
        if !cmds.iter().any(|c| c == &cmd) {
            println!("{} is an invalid command!", cmd);
            println!("CMDS : set <cell_ref>, get <cell_ref>");
            continue;
        }

        let rest = &input[4..];
        let mut parts = rest.splitn(2, char::is_whitespace);

        let raw_ref = parts.next().unwrap_or("").trim(); // cell reference
        let raw_str = parts.next().unwrap_or("").trim(); // rest of the string (value)
        // println!("{} {}", raw_ref, raw_str);

        if let Ok(cell_ref) = CellRef::new(raw_ref.to_owned()) {
            match cmd {
                "set" => match evaluator.set_cell(cell_ref, raw_str.to_owned()) {
                    Ok(_) => println!("Successfully set cell {} to {}.", raw_ref, raw_str),
                    Err(e) => println!("{}", e),
                },
                "get" => match evaluator.get_cell(cell_ref) {
                    Ok(res) => println!("{:?}", res),
                    Err(e) => println!("{}", e),
                },
                _ => {
                    continue; // Impossible
                }
            }
        } else {
            println!("{} is an invalid cell reference!", raw_ref);
            continue;
        }
    }
}
