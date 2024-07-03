use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

// base of the json from the MC api to get the libs
#[derive(Clone, Debug)]
pub struct LibsRoot {
    pub asset_index: AssetIndex,
    pub java_version: u8,
    pub libraries: Vec<Library>,
    pub client: Client,
}

impl LibsRoot {
    pub fn parse_json(json: String) -> Result<LibsRoot, String> {
        let json_object: Value = serde_json::from_str(json.as_str()).unwrap();

        let asset_index: AssetIndex =
            serde_json::from_value(json_object["assetIndex"].clone()).unwrap();
        let java_version: u8 =
            serde_json::from_value(json_object["javaVersion"]["majorVersion"].clone()).unwrap();
        let libraries: Vec<Library> =
            serde_json::from_value(json_object["libraries"].clone()).unwrap();
        let client: Client = serde_json::from_value(json_object["downloads"]["client"].clone()).unwrap();


        Ok(LibsRoot {
            asset_index,
            java_version,
            libraries,
            client,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub total_size: i64,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub downloads: Download,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub artifact: Artifact,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}