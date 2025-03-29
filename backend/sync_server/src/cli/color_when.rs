use std::io::IsTerminal;

use clap::ValueEnum;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl ColorWhen {
    pub fn use_colors(self) -> bool {
        match self {
            ColorWhen::Always => true,
            ColorWhen::Auto => {
                std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
            }
            ColorWhen::Never => false,
        }
    }
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
