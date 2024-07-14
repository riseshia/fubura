use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
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

impl From<aws_sdk_sfn::types::Tag> for ResourceTag {
    fn from(value: aws_sdk_sfn::types::Tag) -> Self {
        ResourceTag {
            key: value.key().unwrap().to_string(),
            value: value.value().unwrap().to_string(),
        }
    }
}

impl Into<aws_sdk_sfn::types::Tag> for ResourceTag {
    fn into(self) -> aws_sdk_sfn::types::Tag {
        aws_sdk_sfn::types::builders::TagBuilder::default()
            .key(self.key)
            .value(self.value)
            .build()
    }
}
