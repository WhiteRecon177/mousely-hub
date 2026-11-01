use rdev::{Button, Key};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ActionKind {
    KeyDown { key: Key },
    KeyUp { key: Key },
    MouseDown { button: Button, x: f64, y: f64 },
    MouseUp { button: Button, x: f64, y: f64 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MacroAction {
    pub delay_ms: u64,
    pub kind: ActionKind,
}