use std::ffi::OsString;

use clap::{Parser, ValueEnum};

/// Server for backing the VaultLink plugin
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(index = 1)]
    pub config_path: Option<OsString>,

    #[arg(
            long,
            require_equals = true,
            value_name = "WHEN",
            num_args = 0..=1,
            default_value_t = ColorWhen::Auto,
            default_missing_value = "always",
            value_enum
        )]
    pub color: ColorWhen,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
