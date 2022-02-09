use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Index {
    objects: HashMap<String, Object>,
}

impl Index {
    /// Get a reference to the index's objects.
    pub(crate) fn objects(&self) -> &HashMap<String, Object> {
        &self.objects
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Object {
    hash: String,
    size: u32,
}

impl Object {
    /// Get a reference to the object's hash.
    pub(crate) fn hash(&self) -> &str {
        self.hash.as_ref()
    }

    /// Get a reference to the object's size.
    pub(crate) fn size(&self) -> u32 {
        self.size
    }
}
