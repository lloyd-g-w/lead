use serde::{Deserialize, Serialize};

use crate::{cell::CellRef, evaluator::Eval};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MsgType {
    Set,
    Get,
    Error,
    Bulk,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeadMsg {
    pub msg_type: MsgType,
    pub cell: Option<CellRef>,
    pub raw: Option<String>,
    pub eval: Option<Eval>,
    pub bulk_msgs: Option<Vec<LeadMsg>>,
}
