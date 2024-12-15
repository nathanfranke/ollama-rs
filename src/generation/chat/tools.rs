use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RawTool {
    Function(Function),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub function: FunctionDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDetails {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FunctionParameter {
    Object(ObjectParameter),
    String(StringParameter),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectParameter {
    pub properties: HashMap<String, FunctionParameter>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringParameter {
    pub description: Option<String>,
}
