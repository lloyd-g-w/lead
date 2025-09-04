mod cell;
mod evaluator;
mod parser;
mod tokenizer;

use futures_util::{SinkExt, StreamExt, TryStreamExt, future};
use log::info;
use std::{env, io::Error};
use tokio::net::{TcpListener, TcpStream};

use crate::{cell::CellRef, evaluator::Evaluator};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).expect("Expected input.");

    // let mut ast = parser::parse(&input).unwrap();
    // println!("{}", ast.pretty());
    let mut evaluator = Evaluator::new();
    // // println!("{}", evaluator.evaluate(input).unwrap());
    // let a1 = CellRef { row: 1, col: 2 };
    // evaluator.set_cell(a1, input).unwrap();
    // println!("{:?}", evaluator.get_cell(a1).unwrap());

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7050".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())

    // println!("CMDS : set <cell_ref>, get <cell_ref>");
    // loop {
    //     let mut input = String::new();
    //     io::stdin().read_line(&mut input).expect("Expected input.");
    //
    //     let cmds = ["set", "get"];
    //     let cmd = &input[0..3];
    //     if !cmds.iter().any(|c| c == &cmd) {
    //         println!("{} is an invalid command!", cmd);
    //         println!("CMDS : set <cell_ref>, get <cell_ref>");
    //         continue;
    //     }
    //
    //     let rest = &input[4..];
    //     let mut parts = rest.splitn(2, char::is_whitespace);
    //
    //     let raw_ref = parts.next().unwrap_or("").trim(); // cell reference
    //     let raw_str = parts.next().unwrap_or("").trim(); // rest of the string (value)
    //     // println!("{} {}", raw_ref, raw_str);
    //
    //     if let Ok(cell_ref) = CellRef::new(raw_ref.to_owned()) {
    //         match cmd {
    //             "set" => match evaluator.set_cell(cell_ref, raw_str.to_owned()) {
    //                 Ok(_) => println!("Successfully set cell {} to {}.", raw_ref, raw_str),
    //                 Err(e) => println!("{}", e),
    //             },
    //             "get" => match evaluator.get_cell(cell_ref) {
    //                 Ok(res) => println!("{:?}", res),
    //                 Err(e) => println!("{}", e),
    //             },
    //             _ => {
    //                 panic!("Impossible.");
    //             }
    //         }
    //     } else {
    //         println!("{} is an invalid cell reference!", raw_ref);
    //         continue;
    //     }
    // }
}

// async fn accept_connection(stream: TcpStream) {
//     let addr = stream
//         .peer_addr()
//         .expect("connected streams should have a peer address");
//     info!("Peer address: {}", addr);
//
//     let ws_stream = tokio_tungstenite::accept_async(stream)
//         .await
//         .expect("Error during the websocket handshake occurred");
//
//     info!("New WebSocket connection: {}", addr);
//
//     let (mut write, mut read) = ws_stream.split();
//
//     // We should not forward messages other than text or binary.
//     while let Some(msg) = read.try_next().await.unwrap_or(None) {
//         if msg.is_text() || msg.is_binary() {
//             if let Err(e) = write
//                 .send(format!("This is a message {}!", msg.to_text().unwrap_or("")).into())
//                 .await
//             {
//                 eprintln!("send error: {}", e);
//                 break;
//             }
//         }
//     }
//
//     info!("Disconnected from {}", addr);
// }

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    // Each connection gets its own evaluator
    let mut evaluator = Evaluator::new();

    while let Some(msg) = read.try_next().await.unwrap_or(None) {
        if msg.is_text() {
            let input = msg.to_text().unwrap_or("").trim().to_string();

            let cmds = ["set", "get"];
            let cmd = &input[0..3.min(input.len())]; // avoid panic on short strings

            if !cmds.iter().any(|c| c == &cmd) {
                let _ = write
                    .send(format!("ERR invalid command: {}", input).into())
                    .await;
                continue;
            }

            let rest = input[4..].trim();
            let mut parts = rest.splitn(2, char::is_whitespace);

            let raw_ref = parts.next().unwrap_or("").trim(); // cell reference
            let raw_str = parts.next().unwrap_or("").trim(); // rest (value)

            if let Ok(cell_ref) = CellRef::new(raw_ref.to_owned()) {
                match cmd {
                    "set" => match evaluator.set_cell(cell_ref.clone(), raw_str.to_owned()) {
                        Ok(eval) => {
                            let _ = write.send(format!("{} {}", raw_ref, eval).into()).await;
                        }
                        Err(e) => {
                            let _ = write.send(format!("ERR {}", e).into()).await;
                        }
                    },
                    "get" => match evaluator.get_cell(cell_ref.clone()) {
                        Ok(res) => {
                            let _ = write
                                .send(format!("{} {}", raw_ref, res.to_string()).into())
                                .await;
                        }
                        Err(e) => {
                            let _ = write.send(format!("ERR {}", e).into()).await;
                        }
                    },
                    _ => {
                        let _ = write.send("ERR impossible".into()).await;
                    }
                }
            } else {
                let _ = write
                    .send(format!("ERR invalid cell reference: {}", raw_ref).into())
                    .await;
            }
        }
    }

    info!("Disconnected from {}", addr);
}
