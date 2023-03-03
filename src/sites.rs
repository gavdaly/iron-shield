use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Site {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub tags: Vec<String>,
}

impl Site {
    pub fn load() -> Vec<Self> {
        vec![
            Self {
                name: "One".to_owned(),
                url: "http://example.com".to_owned(),
                icon: "http://image.png".to_owned(),
                tags: vec!["exterior".to_owned(), "next".to_owned()],
            },
            Self {
                name: "Two".to_owned(),
                url: "http://google.com".to_owned(),
                icon: "icon".to_owned(),
                tags: vec!["exterior".to_owned()],
            },
        ]
    }
}
