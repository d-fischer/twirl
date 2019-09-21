mod string_option;

pub use string_option::StringOption;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
