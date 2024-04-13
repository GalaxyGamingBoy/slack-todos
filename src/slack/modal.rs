use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SlackModal {
    name: String,
    pub data: String,
    pub trigger: String,
}

impl SlackModal {
    pub fn new(name: String, trigger: String) -> Self {
        Self {
            name,
            trigger,
            ..Default::default()
        }
    }

    pub fn load(&mut self) -> &mut Self {
        self.data = fs::read_to_string(format!("./src/modals/{}.modal.json", self.name))
            .expect("Slack modal file wasn't found!");
        self
    }

    pub fn fill(&mut self, args: HashMap<&str, String>) -> &mut Self {
        args.iter().for_each(|arg| {
            let key = format!("{{{{{}}}}}", arg.0);

            self.data = self.data.replace(&key, arg.1);
        });

        self
    }
}
