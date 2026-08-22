use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub name: String,
    pub signature: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<Skin>,
    pub capes: Vec<Cape>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
    pub alias: Option<String>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: Option<String>,
}
