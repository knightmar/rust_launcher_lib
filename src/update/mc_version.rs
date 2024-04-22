use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct GameArguments {
    game: Vec<GameArgument>,
    jvm: Vec<JVMArgument>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameArgument {
    rules: Vec<Rule>,
    value: GameArgumentValue,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum GameArgumentValue {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    action: String,
    #[serde(default)]
    features: Option<HashMap<String, bool>>,
    #[serde(default)]
    os: Option<OSRule>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OSRule {
    #[serde(rename = "name")]
    os_name: Option<String>,
    #[serde(rename = "arch")]
    os_arch: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JVMArgument {
    rules: Vec<Rule>,
    value: JVMArgumentValue,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum JVMArgumentValue {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize)]
struct AssetIndex {
    id: String,
    sha1: String,
    size: u64,
    totalSize: u64,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Downloads {
    client: Artifact,
    client_mappings: Artifact,
    server: Artifact,
    server_mappings: Artifact,
}

#[derive(Debug, Deserialize, Serialize)]
struct Artifact {
    sha1: String,
    size: u64,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JavaVersion {
    component: String,
    majorVersion: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct Library {
    downloads: LibraryDownloads,
    name: String,
    #[serde(default)]
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LibraryDownloads {
    artifact: Artifact,
}

#[derive(Debug, Deserialize, Serialize)]
struct Logging {
    client: ClientLogging,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClientLogging {
    argument: String,
    file: LogFile,
    #[serde(rename = "type")]
    log_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogFile {
    id: String,
    sha1: String,
    size: u64,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct MinecraftManifest {
    arguments: GameArguments,
    assetIndex: AssetIndex,
    assets: String,
    complianceLevel: u8,
    downloads: Downloads,
    id: String,
    javaVersion: JavaVersion,
    libraries: Vec<Library>,
    logging: Logging,
    mainClass: String,
    minimumLauncherVersion: u8,
    releaseTime: String,
    time: String,
    #[serde(rename = "type")]
    release_type: String,
}