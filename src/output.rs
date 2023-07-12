use crate::{Args, Result};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use clap::ValueEnum;

pub const SEPARATOR_THAT_WONT_COLIDE_FOR_SURE: char = '\u{1F}';

pub struct Output<'args> {
    casing: Casing,
    separator: char,
    spawn: Option<&'args OsStr>,
}

impl<'args> Output<'args> {
    fn new(casing: Casing, separator: char, spawn: Option<&'args OsStr>) -> Self {
        Self {
            casing,
            separator,
            spawn,
        }
    }

    pub fn from_args(args: &'args Args) -> Self {
        Self::new(args.casing, args.separator, args.spawn.as_deref())
    }

    pub fn print(&self, key: &str, value: &str) -> Result<()> {
        // Replace key with the given separator
        let key = key.replace(
            SEPARATOR_THAT_WONT_COLIDE_FOR_SURE,
            &self.separator.to_string(),
        );
        let key = self.casing.apply(&key);
        // Print the thing to stdout
        print_env(&key, value);
        // Spawn the binary if output path was provided
        if let Some(output) = &self.spawn {
            spawn_binary_at(&key, value, output)?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, ValueEnum, Copy)]
pub enum Casing {
    #[default]
    Insensitive,
    Lower,
    Upper,
}

impl Casing {
    fn apply(self, value: &str) -> String {
        match self {
            Casing::Insensitive => value.to_owned(),
            Casing::Lower => value.to_lowercase(),
            Casing::Upper => value.to_uppercase(),
        }
    }
}

impl std::fmt::Display for Casing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Casing::Insensitive => write!(f, "insensitive"),
            Casing::Lower => write!(f, "lower"),
            Casing::Upper => write!(f, "upper"),
        }
    }
}

/// Create a binary template with the given shell and value
fn binary_template(shell: &str, value: &str) -> String {
    format!(
        r#"
#!{shell}
echo {value}
    "#
    )
}

/// drwxr-xr-x
const EXECUTABLE_UNIX_MODE_BITS: u32 = 0o0_040_755;

/// Create a binary file with the given key and value
fn spawn_binary_at(key: &str, value: &str, output: &OsStr) -> Result<()> {
    let shell = get_shell();
    let template = binary_template(&shell, value);

    let bin = fs::File::create(output)?;
    // Set executable permissions
    let mut perms = bin.metadata()?.permissions();
    perms.set_mode(EXECUTABLE_UNIX_MODE_BITS);
    bin.set_permissions(perms)?;
    fs::write(key, template)?;
    Ok(())
}

/// Print the environment variables to stdout
fn print_env(key: &str, value: &str) {
    println!("{key}={value}; export {key};");
}

/// The laziest way to get your system shell
fn get_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_owned())
}
