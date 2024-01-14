pub type AppErr = Box<dyn std::error::Error>;
pub type AppResult<T> = std::result::Result<T, AppErr>;
