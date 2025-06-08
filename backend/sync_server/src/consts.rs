use std::time::Duration;

pub const DEFAULT_CONFIG_PATH: &str = "config.yml";

pub const DEFAULT_DATABASES_DIRECTORY_PATH: &str = "databases";
pub const DEFAULT_MAX_CONNECTIONS_PER_VAULT: u32 = 12;
pub const DEFAULT_CURSOR_TIMEOUT: Duration = Duration::from_secs(60);

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 3000;
pub const DEFAULT_MAX_BODY_SIZE_MB: usize = 4096;
pub const DEFAULT_RESPONSE_TIMEOUT_SECONDS: u64 = 60;
pub const DEFAULT_MAX_CLIENTS_PER_VAULT: usize = 256;
