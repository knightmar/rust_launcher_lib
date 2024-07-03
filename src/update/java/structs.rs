use serde_derive::Deserialize;
use serde_derive::Serialize;

pub type ZuluRoot = Vec<Version>;

// struct used to parse the json from the azul api
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub package_uuid: String,
    pub name: String,
    pub java_version: Vec<i64>,
    pub openjdk_build_number: i64,
    pub latest: bool,
    pub download_url: String,
    pub product: String,
    pub distro_version: Vec<i64>,
    pub availability_type: String,
}
