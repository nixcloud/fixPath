
const VERSION_OPTION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub static VERSION: &str = match VERSION_OPTION {
    Some(version) => version,
    None => "unknown version",
};

const NAME_OPTION: Option<&str> = option_env!("CARGO_PKG_NAME");

pub static NAME: &str = match crate::cargo_env::NAME_OPTION {
    Some(name) => name,
    None => "unknown program name",
};