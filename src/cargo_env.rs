use std::env;

pub fn get_executable_name() -> String {
    env::current_exe()
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown program name".into())
}

const VERSION_OPTION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub static VERSION: &str = match VERSION_OPTION {
    Some(version) => version,
    None => "unknown version",
};