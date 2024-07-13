use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ResourceTag {
    pub key: String,
    pub value: String,
}

impl From<std::collections::HashMap<::std::string::String, ::std::string::String>> for ResourceTag {
    fn from(
        value: std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) -> Self {
        ResourceTag {
            key: value.get("key").unwrap().to_string(),
            value: value.get("value").unwrap().to_string(),
        }
    }
}
