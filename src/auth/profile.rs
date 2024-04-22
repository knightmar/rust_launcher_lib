use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Skin {
    id: String,
    state: String,
    url: String,
    textureKey: String,
    variant: String,
}
impl Skin {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn textureKey(&self) -> &str {
        &self.textureKey
    }
    pub fn variant(&self) -> &str {
        &self.variant
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cape {
    id: String,
    state: String,
    url: String,
    alias: String,
}

impl Cape {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn alias(&self) -> &str {
        &self.alias
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: String,
    name: String,
    skins: Vec<Skin>,
    capes: Vec<Cape>,
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
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn skins(&self) -> &Vec<Skin> {
        &self.skins
    }
    pub fn capes(&self) -> &Vec<Cape> {
        &self.capes
    }
    pub fn profileActions(&self) -> &serde_json::Value {
        &self.profileActions
    }
}