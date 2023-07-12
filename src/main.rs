#![warn(clippy::all, clippy::pedantic, clippy::cargo)]

use std::ffi::OsString;
use std::io;

use clap::Parser;

mod dotenv;
mod error;
mod input;
mod json;
mod output;

use error::{Error, Result};
use input::Input;
use output::{Casing, Output};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input file
    #[arg(name = "FILE")]
    file: Option<OsString>,
    /// Spawn tiny binaries at `PATH` with named with the given `key` paths that when executed return `value`.
    ///
    /// Use '.' to spawn at the current directory
    ///
    /// Use 'temp' to spawn at your system temporary directory
    #[arg(short = 'o', long, name = "PATH")]
    spawn: Option<OsString>,
    /// Match keys with the given template
    ///
    /// json:
    ///  --match '{
    ///     "a": $FOO,
    ///     "b": [_, _, $BAR],
    ///     "c": { "d": $BORG },
    ///     ...
    ///  }'
    ///
    /// .env:
    /// --match 'a=$BAZ;b=$BURG;...'
    #[arg(short, long, name = "TEMPLATE")]
    r#match: Option<String>,
    /// Separator used to chain nesting keys.
    /// (Not applicable for .env files)
    #[arg(short, long, default_value_t = '.')]
    separator: char,
    /// Case sensitivity for the output keys.
    #[arg(short, long, name = "CASING", default_value_t = Casing::Insensitive)]
    casing: Casing,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = Input::from_args(&args)?;
    let output = Output::from_args(&args);

    match input.ext {
        ext if ext == "json" => {
            let reader = io::BufReader::new(input.reader);
            let data: serde_json::Value = serde_json::from_reader(reader)?;

            // Flattened json data
            let mut flattened = json::Map::new();
            json::dfs_flatten(&data, &mut flattened, "");

            // Flattened matches
            let maybe_matches = if let Some(ref matches) = args.r#match {
                let matches = json::into_fixed(matches)?;
                let mut flattened_to_match = json::Map::new();
                json::dfs_flatten(&matches, &mut flattened_to_match, "");
                Some(flattened_to_match)
            } else {
                None
            };

            for (key, value) in flattened {
                // We are matching the keys with the given template
                if let Some(matches) = &maybe_matches {
                    // If the key is not in the matches, skip it
                    if matches.contains_key(&key) {
                        let var_name = matches
                            .get(&key)
                            .ok_or(Error::InvalidMatchTemplate(key))?
                            .to_string()
                            // Remove surrounding quotes
                            .replace('\"', "");
                        output.print(&var_name, value.as_str().unwrap_or_default())?;
                    }
                } else {
                    output.print(&key, value.as_str().unwrap_or_default())?;
                }
            }
        }
        ext if ext == "yaml" || ext == "yml" => {
            unimplemented!("YAML support is not implemented yet")
        }
        ext if ext == "toml" => {
            unimplemented!("TOML support is not implemented yet")
        }
        ext if ext == "xml" => {
            unimplemented!("XML support is not implemented yet")
        }
        // Assuming it's a .env like file
        _ => {
            let lines = dotenv::from_reader(input.reader);
            let maybe_matches = if let Some(ref matches) = args.r#match {
                let matches = dotenv::into_fixed(matches);
                let mut map = dotenv::Map::new();
                let matched_vars = dotenv::from_str(&matches)?;
                for kv in matched_vars {
                    let (key, value) = kv?;
                    map.insert(key, value);
                }
                Some(map)
            } else {
                None
            };

            for line in lines {
                let (key, value) = line?;
                if let Some(matches) = &maybe_matches {
                    if matches.contains_key(&key) {
                        let var_name = matches
                            .get(&key)
                            .ok_or(Error::InvalidMatchTemplate(key))?
                            .to_string()
                            // Remove surrounding quotes
                            .replace('\"', "");
                        output.print(&var_name, &value.to_string())?;
                    }
                } else {
                    output.print(&key, &value.to_string())?;
                }
            }
        }
    }
    Ok(())
}
