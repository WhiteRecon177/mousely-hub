use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RemapAction {
    Normal,
    Disabled,
    Keys { keys: Vec<String> },
    Macro { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mapping {
    pub button: String,
    pub action: RemapAction,
}