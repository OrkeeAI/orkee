use std::env;

pub struct Config {
    pub port: u16,
    pub cors_origin: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");
            
        let cors_origin = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        Config {
            port,
            cors_origin,
        }
    }
}