use serde::{Deserialize, Serialize};

use crate::{cell::CellRef, tokenizer::Literal};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgType {
    Set,
    Get,
    Error,
    Bulk,
}

#[derive(Serialize, Deserialize)]
pub struct LeadMsg {
    pub msg_type: MsgType,
    pub cell: Option<CellRef>,
    pub raw: Option<String>,
    pub eval: Option<Literal>,
    pub bulk_msgs: Option<Vec<LeadMsg>>,
}
