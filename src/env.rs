use serde::Serialize;
use std::env;
use std::fmt::{format, Debug};

#[derive(Debug, Serialize)]
pub struct Env {
    env: AppEnv,
    debug: bool,
}

impl Env {
    pub(crate) fn init() -> Env {
        dotenvy::dotenv().expect("Failed to load .env");
        Env {
            env: env::var("APP_ENV").expect("APP_ENV not set").into(),
            debug: Self::parse_bool("APP_DEBUG"),
        }
    }

    pub fn env(&self) -> &AppEnv {
        &self.env
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    fn parse_bool(handle: &str) -> bool {
        let var = env::var(handle)
            .expect(format!("{handle} not set in .env file").as_str())
            .to_ascii_lowercase();
        match var.as_str() {
            "true" => true,
            "false" => false,
            _ => panic!("Invalid boolean value for {handle}"),
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum AppEnv {
    Local,
    Production,
    Other(String),
}

impl From<String> for AppEnv {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "local" => AppEnv::Local,
            "production" => AppEnv::Production,
            _ => AppEnv::Other(value),
        }
    }
}
