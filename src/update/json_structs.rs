use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    pub releaseTime: String,
    pub sha1: String,
    pub complianceLevel: i32,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct UniversalVersionJson {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: Option<String>,
    pub time: Option<String>,
    #[serde(rename = "releaseTime")]
    pub release_time: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<i32>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,

    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,

    pub downloads: Option<Downloads>,

    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<Arguments>,

    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,

    pub libraries: Vec<Library>,

    pub logging: Option<Logging>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(rename = "totalSize")]
    pub total_size: Option<i64>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Downloads {
    pub client: Option<Artifact>,
    pub server: Option<Artifact>,
    #[serde(rename = "windows_server")]
    pub windows_server: Option<Artifact>,
    #[serde(rename = "client_mappings")]
    pub client_mappings: Option<Artifact>,
    #[serde(rename = "server_mappings")]
    pub server_mappings: Option<Artifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Arguments {
    pub game: Option<Vec<Argument>>,
    pub jvm: Option<Vec<Argument>>,
    #[serde(rename = "default-user-jvm")]
    pub default_user_jvm: Option<Vec<Argument>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Object(ArgumentObject),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArgumentObject {
    pub rules: Option<Vec<Rule>>,
    pub value: Value,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<Os>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Os {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
    #[serde(rename = "versionRange")]
    pub version_range: Option<VersionRange>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<HashMap<String, String>>,
    pub rules: Option<Vec<Rule>>,
    pub extract: Option<Extract>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extract {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Logging {
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub log_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

impl<'de> Deserialize<'de> for UniversalVersionJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let mut result = UniversalVersionJson::default();

        if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
            result.id = id.to_string();
        }

        if let Some(vt) = value.get("type").and_then(|v| v.as_str()) {
            result.version_type = Some(vt.to_string());
        }

        if let Some(t) = value.get("time").and_then(|v| v.as_str()) {
            result.time = Some(t.to_string());
        }

        if let Some(rt) = value.get("releaseTime").and_then(|v| v.as_str()) {
            result.release_time = Some(rt.to_string());
        }

        if let Some(mc) = value.get("mainClass").and_then(|v| v.as_str()) {
            result.main_class = Some(mc.to_string());
        }

        if let Some(mlv) = value.get("minimumLauncherVersion").and_then(|v| v.as_i64()) {
            result.minimum_launcher_version = Some(mlv as i32);
        }

        if let Some(cl) = value.get("complianceLevel").and_then(|v| v.as_i64()) {
            result.compliance_level = Some(cl as i32);
        }

        if let Some(assets) = value.get("assets").and_then(|v| v.as_str()) {
            result.assets = Some(assets.to_string());
        }

        if let Some(ai) = value.get("assetIndex") {
            if let Ok(asset_index) = serde_json::from_value(ai.clone()) {
                result.asset_index = Some(asset_index);
            }
        }

        if let Some(d) = value.get("downloads") {
            if let Ok(downloads) = serde_json::from_value(d.clone()) {
                result.downloads = Some(downloads);
            }
        }

        if let Some(ma) = value.get("minecraftArguments").and_then(|v| v.as_str()) {
            result.minecraft_arguments = Some(ma.to_string());
        }

        if let Some(args) = value.get("arguments") {
            if let Ok(arguments) = serde_json::from_value(args.clone()) {
                result.arguments = Some(arguments);
            }
        }

        if let Some(jv) = value.get("javaVersion") {
            if let Ok(java_version) = serde_json::from_value(jv.clone()) {
                result.java_version = Some(java_version);
            }
        }

        if let Some(libs) = value.get("libraries").and_then(|v| v.as_array()) {
            for lib in libs {
                if let Ok(library) = serde_json::from_value(lib.clone()) {
                    result.libraries.push(library);
                }
            }
        }

        if let Some(logging) = value.get("logging") {
            if let Ok(log) = serde_json::from_value(logging.clone()) {
                result.logging = Some(log);
            }
        }

        let known_fields = [
            "id",
            "type",
            "time",
            "releaseTime",
            "mainClass",
            "minimumLauncherVersion",
            "complianceLevel",
            "assets",
            "assetIndex",
            "downloads",
            "minecraftArguments",
            "arguments",
            "javaVersion",
            "libraries",
            "logging",
        ];

        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                if !known_fields.contains(&key.as_str()) {
                    result.extra.insert(key.clone(), val.clone());
                }
            }
        }

        Ok(result)
    }
}

impl UniversalVersionJson {
    pub fn uses_structured_arguments(&self) -> bool {
        self.arguments.is_some()
    }

    pub fn get_game_arguments(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(arguments) = &self.arguments {
            if let Some(game_args) = &arguments.game {
                for arg in game_args {
                    match arg {
                        Argument::String(s) => args.push(s.clone()),
                        Argument::Object(obj) => {
                            if let Some(value) = obj.value.as_str() {
                                args.push(value.to_string());
                            } else if let Some(values) = obj.value.as_array() {
                                for v in values {
                                    if let Some(s) = v.as_str() {
                                        args.push(s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        args
    }

    pub fn get_jvm_arguments(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(arguments) = &self.arguments {
            if let Some(jvm_args) = &arguments.jvm {
                for arg in jvm_args {
                    match arg {
                        Argument::String(s) => args.push(s.clone()),
                        Argument::Object(obj) => {
                            if let Some(value) = obj.value.as_str() {
                                args.push(value.to_string());
                            } else if let Some(values) = obj.value.as_array() {
                                for v in values {
                                    if let Some(s) = v.as_str() {
                                        args.push(s.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        args
    }

    pub fn get_minecraft_arguments_string(&self) -> Option<&str> {
        self.minecraft_arguments.as_deref()
    }

    pub fn get_client_url(&self) -> Option<&str> {
        self.downloads
            .as_ref()
            .and_then(|d| d.client.as_ref())
            .map(|a| a.url.as_str())
    }

    pub fn get_server_url(&self) -> Option<&str> {
        self.downloads
            .as_ref()
            .and_then(|d| d.server.as_ref())
            .map(|a| a.url.as_str())
    }
}
