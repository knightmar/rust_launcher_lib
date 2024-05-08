use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Version {
    id: String,
    r#type: String,
    url: String,
    time: String,
    releaseTime: String,
    sha1: String,
    complianceLevel: u8,
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.r#type == other.r#type
            && self.url == other.url
            && self.time == other.time
            && self.releaseTime == other.releaseTime
            && self.sha1 == other.sha1
            && self.complianceLevel == other.complianceLevel
    }
}

impl Version {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn r#type(&self) -> &str {
        &self.r#type
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn time(&self) -> &str {
        &self.time
    }
    pub fn releaseTime(&self) -> &str {
        &self.releaseTime
    }
    pub fn sha1(&self) -> &str {
        &self.sha1
    }
    pub fn complianceLevel(&self) -> u8 {
        self.complianceLevel
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Versions {
    latest: Latest,
    versions: Vec<Version>,
}

impl Versions {
    pub fn latest(&self) -> &Latest {
        &self.latest
    }
    pub fn versions(&self) -> &Vec<Version> {
        &self.versions
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Latest {
    release: String,
    snapshot: String,
}

impl Latest {
    pub fn release(&self) -> &str {
        &self.release
    }
    pub fn snapshot(&self) -> &str {
        &self.snapshot
    }
}
