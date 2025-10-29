use std::path::Path;

use crate::env::EnvVars;
// use dotenv::dotenv;
use envconfig::Envconfig;

pub fn load_env_vars() -> EnvVars {
    // if dotenv().ok().is_none() {
    let env_path;
    if cfg!(debug_assertions) {
        env_path = Path::new("./.env.dev");
    } else {
        env_path = Path::new("./.env");
    }

    if let Err(e) = dotenv::from_path(env_path) {
        panic!("Failed to load {} file: {}", env_path.display(), e);
    }

    let env_vars = EnvVars::init_from_env();
    match env_vars {
        Ok(e) => return e,
        Err(e) => panic!("failed to serialize .env: {}", e),
    }
}
