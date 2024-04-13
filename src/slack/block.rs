use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackBlock {
    name: String,
    pub data: String,
}

impl SlackBlock {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn load(&mut self) -> &mut Self {
        self.data = fs::read_to_string(format!("./src/blocks/{}.block.json", self.name))
            .expect("Slack block file not found!");
        self
    }

    pub fn fill(&mut self, args: HashMap<&str, String>) -> &mut Self {
        args.iter().for_each(|arg| {
            let key = format!("{{{{{}}}}}", arg.0);

            self.data = self.data.replace(&key, arg.1);
        });

        self
    }

    pub fn trim(&mut self) -> &mut Self {
        let data: Value =
            serde_json::from_str(&self.data).expect("Slack block isn't valid JSON data");
        self.data = data["blocks"].to_string();

        self
    }
}

impl Into<Value> for SlackBlock {
    fn into(self) -> Value {
        serde_json::from_str(&self.data).unwrap_or_default()
    }
}
