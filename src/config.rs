use anyhow::{Context, Result};
use dirs::config_dir;
use std::{fs, path::PathBuf};

pub const APP_ID: Option<&str> = option_env!("APP_ID");
pub const GRESOURCE_PATH: Option<&str> = option_env!("GRESOURCE_PATH");
pub const RESOURCES_FILE: Option<&str> = option_env!("RESOURCES_FILE");
pub const APP_NAME: Option<&str> = option_env!("APP_NAME");
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn app_id() -> &'static str {
    APP_ID.expect("APP_ID env var not set at compile time")
}

pub fn gresource_path() -> &'static str {
    GRESOURCE_PATH.expect("GRESOURCE_PATH env var not set at compile time")
}

pub fn app_name() -> &'static str {
    APP_NAME.expect("APP_NAME env var not set at compile time")
}

pub fn resources_file() -> &'static str {
    RESOURCES_FILE.expect("RESOURCES_FILE env var not set at compile time")
}

pub fn app_config_dir() -> Result<PathBuf> {
    let path = config_dir()
        .context("Could not determine config directory")?
        .join(PACKAGE_NAME);
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create config directory: {}", path.display()))?;
    Ok(path)
}
