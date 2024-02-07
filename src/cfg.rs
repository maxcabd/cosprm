use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CostumeAddConfig {
    pub costumes: Vec<CostumeConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CostumeConfig {
    pub model_index: i32,
    pub characode: String,
    pub modelcode: String,
    pub iconcode: String,
    pub cha_id: String,
    pub char_name: String,
    pub costume_id: String,
    pub costume_name: String,
    pub color_count: i32,
    pub has_costume_break: bool,
}

impl CostumeAddConfig {
    pub fn read_cfg(filepath: &str) -> Self {
        let json_str = std::fs::read_to_string(filepath).unwrap();
        serde_json::from_str(&json_str).unwrap()
    }
}
