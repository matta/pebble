use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    pub id: String,
    pub closed_at: Option<String>,
}

fn main() {
    let json = r#"{"id": "123"}"#;
    let issue: Result<Issue, _> = serde_json::from_str(json);
    println!("{:?}", issue);
}
