use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RemapAction {
    Normal,
    Disabled,
    Keys { keys: Vec<String> },
    Macro { id: String },
}

#[derive(Deserialize, Clone, Debug)]
pub struct Mapping {
    pub button: String,
    pub action: RemapAction,
}