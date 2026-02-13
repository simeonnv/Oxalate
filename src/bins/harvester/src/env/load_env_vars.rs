use std::path::Path;

use crate::env::EnvVars;
// use dotenv::dotenv;
use envconfig::Envconfig;

pub fn load_env_vars() -> EnvVars {
    let env_path = if cfg!(debug_assertions) {
        Path::new("./.env.dev")
    } else {
        Path::new("./.env")
    };

    if let Err(e) = dotenv::from_path(env_path) {
        eprintln!("Failed to load {} file: {}", env_path.display(), e);
    }

    let env_vars = EnvVars::init_from_env();
    match env_vars {
        Ok(e) => e,
        Err(e) => panic!("failed to load env vars: {}", e),
    }
}
