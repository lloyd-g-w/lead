mod cell;
mod common;
mod evaluator;
mod grid;
mod messages;
mod parser;
mod tokenizer;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use log::info;
use std::{env, io::Error};
use tokio::net::{TcpListener, TcpStream};

use crate::{
    grid::Grid,
    messages::{LeadMsg, MsgType},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

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
}

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
    let mut grid = Grid::new();

    while let Some(msg) = read.try_next().await.unwrap_or(None) {
        if msg.is_text() {
            let input = msg.to_text().unwrap();

            if let Ok(req) = serde_json::from_str::<LeadMsg>(&input) {
                match req.msg_type {
                    MsgType::Set => {
                        let Some(cell_ref) = req.cell else { continue };
                        let Some(raw) = req.raw else { continue };
                        // let config = req.eval_config.unwrap_or_default();

                        match grid.update_cell(cell_ref.clone(), raw.to_owned()) {
                            Ok(updates) => {
                                let mut msgs = Vec::new();

                                for update in &updates {
                                    if let Ok(cell) = grid.get_cell(*update) {
                                        msgs.push(LeadMsg {
                                            msg_type: MsgType::Set,
                                            cell: Some(*update),
                                            raw: Some(cell.raw()),
                                            eval: Some(cell.eval()),
                                            bulk_msgs: None,
                                            eval_config: None,
                                        });
                                    }
                                }

                                if msgs.len() == 1 {
                                    let _ = write
                                        .send(serde_json::to_string(&msgs.get(0)).unwrap().into())
                                        .await;
                                } else if msgs.len() > 1 {
                                    let msg = LeadMsg {
                                        cell: None,
                                        raw: None,
                                        eval: None,
                                        eval_config: None,
                                        bulk_msgs: Some(msgs),
                                        msg_type: MsgType::Bulk,
                                    };

                                    let _ = write
                                        .send(serde_json::to_string(&msg).unwrap().into())
                                        .await;
                                }
                            }
                            Err(e) => {
                                let res = LeadMsg {
                                    msg_type: MsgType::Error,
                                    cell: Some(cell_ref),
                                    raw: Some(e.to_string()),
                                    eval: None,
                                    eval_config: None,
                                    bulk_msgs: None,
                                };
                                let _ = write
                                    .send(serde_json::to_string(&res).unwrap().into())
                                    .await;
                            }
                        }
                    }
                    MsgType::Eval => {
                        let Some(cell_ref) = req.cell else { continue };
                        let Some(raw) = req.raw else { continue };

                        let eval = grid.quick_eval(raw.to_owned());

                        let msg = LeadMsg {
                            msg_type: MsgType::Eval,
                            cell: Some(cell_ref),
                            raw: Some(raw),
                            eval: Some(eval),
                            bulk_msgs: None,
                            eval_config: None,
                        };

                        let _ = write
                            .send(serde_json::to_string(&msg).unwrap().into())
                            .await;
                    }
                    _ => {
                        continue; // handle other cases
                    }
                }
            } else {
                continue;
            }
        }
    }

    info!("Disconnected from {}", addr);
}
