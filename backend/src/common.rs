use serde::{Deserialize, Serialize};

// Example usage:
//
// title: "Evaluation error."
// desc: "Function ADD requires numeric type arguments."
// code: TypeErr
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct LeadErr {
    pub title: String,
    pub desc: String,
    pub code: LeadErrCode,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum LeadErrCode {
    DivZero,
    TypeErr,
    Syntax,
    Server,
    Unsupported,
    Invalid,
    Ref,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Literal {
    Number(f64),
    Boolean(bool),
    String(String),
}
