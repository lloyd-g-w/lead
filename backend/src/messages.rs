use serde::{Deserialize, Serialize};

use crate::{cell::CellRef, evaluator::Eval};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MsgType {
    Set,
    Eval,
    Get,
    Error,
    Bulk,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvalConfig {
    pub do_propagation: bool,
    pub force_propagation: bool,
}

impl Default for EvalConfig {
    fn default() -> Self {
        EvalConfig {
            do_propagation: true,
            force_propagation: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeadMsg {
    pub msg_type: MsgType,
    pub cell: Option<CellRef>,
    pub raw: Option<String>,
    pub eval: Option<Eval>,
    pub eval_config: Option<EvalConfig>,
    pub bulk_msgs: Option<Vec<LeadMsg>>,
}
