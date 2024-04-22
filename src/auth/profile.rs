use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub textureKey: String,
    pub variant: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub skins: Vec<Skin>,
    pub capes: Vec<Cape>,
    profileActions: serde_json::Value,
}

impl User {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            skins: vec![],
            capes: vec![],
            profileActions: Value::from(()),
        }
    }
}