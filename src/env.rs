use serde::Serialize;
use std::env;
use std::fmt::Debug;

#[derive(Debug, Serialize)]
pub struct Env {
    env: AppEnv,
    debug: bool,
}

impl Env {
    pub(crate) fn init() -> Env {
        dotenvy::dotenv().expect("Failed to load .env");
        Env {
            env: env::var("APP_ENV").expect("APP_ENV not set.").into(),
            debug: Self::parse_bool(env::var("APP_DEBUG").expect("APP_DEBUG not set.")),
        }
    }

    pub fn env(&self) -> &AppEnv {
        &self.env
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    fn parse_bool(string: String) -> bool {
        match string.to_ascii_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => panic!(),
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
