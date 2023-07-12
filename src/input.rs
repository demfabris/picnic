use std::io::{Cursor, Read};
use std::{fs, io, path};

use crate::dotenv;
use crate::{Args, Error, Result};

// Simply try to deserialize to our supported formats and see if it works
fn guess_input_format(input: &str) -> Result<&'static str> {
    let json = serde_json::from_str::<serde_json::Value>(input);
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(input);
    let dotenv = dotenv::from_str(input);
    match (json, yaml, dotenv) {
        (Ok(_), _, _) => Ok("json"),
        (_, Ok(_), _) => Ok("yaml"),
        (_, _, Ok(_)) => Ok("dotenv"),
        _ => Err(Error::InvalidInputFormat(
            "Unsupported data format from stdin".to_owned(),
        )),
    }
}

pub struct Input {
    pub ext: String,
    pub reader: Box<dyn Read>,
}

impl Input {
    pub fn from_args(args: &Args) -> Result<Self> {
        if let Some(ref file) = args.file {
            let meta = fs::metadata(file)?;
            // Sanity check
            if !meta.is_file() {
                return Err(Error::NotAFile);
            }

            let ext = path::Path::new(file)
                .extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            Ok(Self {
                ext,
                reader: Box::new(fs::File::open(file)?),
            })
        } else {
            // If the user didn't provide a file, we'll try to read from stdin
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let ext = guess_input_format(&input)?.to_owned();
            Ok(Self {
                ext,
                reader: Box::new(Cursor::new(input)),
            })
        }
    }
}
