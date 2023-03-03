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
                name: "OpenWeatherMap".to_owned(),
                url: "https://openweathermap.org/api".to_owned(),
                icon: "http://openweathermap.org/themes/openweathermap/assets/vendor/owm/img/icons/logo_60x60.png".to_owned(),
                tags: vec!["weather".to_owned(), "project".to_owned()],
            },
            Self {
                name: "Bing".to_owned(),
                url: "https://bing.com".to_owned(),
                icon: "http://openweathermap.org/themes/openweathermap/assets/vendor/owm/img/icons/logo_60x60.png".to_owned(),
                tags: vec!["search".to_owned(), "project".to_owned()],
            },
        ]
    }
}
