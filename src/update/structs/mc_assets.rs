use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object {
    hash: String,
    size: i32,
}

impl Object {
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn size(&self) -> i32 {
        self.size
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetsRoot {
                //   path  ; (hash + size)
    objects: HashMap<String, Object>,
}

impl AssetsRoot {
    pub fn objects(&self) -> &HashMap<String, Object> {
        &self.objects
    }
}
