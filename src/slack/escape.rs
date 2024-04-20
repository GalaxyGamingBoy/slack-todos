pub struct SlackEscape {
    pub id: String,
    pub display: String,
}

impl SlackEscape {
    pub fn from(s: String) -> Self {
        let mut chars = s.chars();
        chars.next();
        chars.next();
        chars.next_back();

        let s: Vec<&str> = chars.as_str().split("|").collect();
        Self {
            id: s[0].to_string(),
            display: s[1].to_string(),
        }
    }
}
