use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub hp: i32,
    pub max_hp: i32,
    pub unreserved_hp: i32,
    pub es: i32,
    pub max_es: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mana {
    pub mana: i32,
    pub max_mana: i32,
    pub unreserved_mana: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockWhenVariable {
    Life,
    Mana,
    Es,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockWhenCondition {
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = ">")]
    Gt,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockClickVariable {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockScrollVariable {
    Up,
    Down,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BlockDefinition {
    // Events
    Stat {
        variable: BlockWhenVariable,
        condition: BlockWhenCondition,
        value: String,
    },
    Key {
        value: String,
    },
    Mouse {
        variable: BlockClickVariable,
    },
    // Actions
    Press {
        value: String,
    },
    Type {
        value: String,
    },
    Disconnect,
    Click {
        variable: BlockClickVariable,
    },
    Scroll {
        variable: BlockScrollVariable,
    },
    // Timing
    Cooldown {
        value: String,
    },
    Delay {
        value: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleDefinition {
    name: String,
    blocks: Vec<BlockDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileDefinition {
    name: String,
    rules: Vec<BlockDefinition>,
}
