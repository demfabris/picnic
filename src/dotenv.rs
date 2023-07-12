//! Parser for .env files
/// credits: [dotenvy](https://docs.rs/dotenv/0.15.0/src/dotenv/parse.rs.html)
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::io::{BufReader, Lines};

use regex::Regex;

use crate::error::{Error, Result};

pub type Map = BTreeMap<String, String>;

pub fn into_fixed(value: &str) -> String {
    let re = Regex::new(r"\$\s*\b(\w+)\b").unwrap();
    let corrected_str = re.replace_all(value, |caps: &regex::Captures| caps[1].to_string());
    // Replace semicolons with newlines
    corrected_str.replace(';', "\n")
}

#[allow(clippy::unnecessary_wraps)]
pub fn from_str(input: &str) -> Result<parser::Iter<&[u8]>> {
    Ok(parser::Iter::new(input.as_bytes()))
}

pub fn from_reader(read: impl Read) -> parser::Iter<impl Read> {
    parser::Iter::new(read)
}

mod parser {
    use super::{BTreeMap, BufRead, BufReader, Error, Lines, Read, Result};
    pub struct Iter<R: Read> {
        lines: Lines<BufReader<R>>,
        substitution_data: BTreeMap<String, Option<String>>,
    }

    impl<R: Read> Iter<R> {
        pub fn new(read: R) -> Iter<R> {
            Iter {
                lines: BufReader::new(read).lines(),
                substitution_data: BTreeMap::new(),
            }
        }
    }

    impl<R: Read> Iterator for Iter<R> {
        type Item = Result<(String, String)>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let line = match self.lines.next() {
                    Some(Ok(line)) => line,
                    Some(Err(err)) => return Some(Err(Error::Io(err))),
                    None => return None,
                };

                match parse_line(&line, &mut self.substitution_data) {
                    Ok(Some(result)) => return Some(Ok(result)),
                    Ok(None) => {}
                    Err(err) => return Some(Err(err)),
                }
            }
        }
    }

    type ParsedLine = Result<Option<(String, String)>>;

    fn parse_line(
        line: &str,
        substitution_data: &mut BTreeMap<String, Option<String>>,
    ) -> ParsedLine {
        let mut parser = LineParser::new(line, substitution_data);
        parser.parse_line()
    }

    struct LineParser<'a> {
        original_line: &'a str,
        substitution_data: &'a mut BTreeMap<String, Option<String>>,
        line: &'a str,
        pos: usize,
    }

    impl<'a> LineParser<'a> {
        fn new(
            line: &'a str,
            substitution_data: &'a mut BTreeMap<String, Option<String>>,
        ) -> LineParser<'a> {
            LineParser {
                original_line: line,
                substitution_data,
                line: line.trim_end(), // we don’t want trailing whitespace
                pos: 0,
            }
        }

        fn err(&self) -> Error {
            Error::LineParse(self.original_line.into(), self.pos)
        }

        fn parse_line(&mut self) -> ParsedLine {
            self.skip_whitespace();
            // if its an empty line or a comment, skip it
            if self.line.is_empty() || self.line.starts_with('#') {
                return Ok(None);
            }

            let mut key = self.parse_key()?;
            self.skip_whitespace();

            // export can be either an optional prefix or a key itself
            if key == "export" {
                // here we check for an optional `=`, below we throw directly when it’s not found.
                if self.expect_equal().is_err() {
                    key = self.parse_key()?;
                    self.skip_whitespace();
                    self.expect_equal()?;
                }
            } else {
                self.expect_equal()?;
            }
            self.skip_whitespace();

            if self.line.is_empty() || self.line.starts_with('#') {
                self.substitution_data.insert(key.clone(), None);
                return Ok(Some((key, String::new())));
            }

            let parsed_value = parse_value(self.line, self.substitution_data)?;
            self.substitution_data
                .insert(key.clone(), Some(parsed_value.clone()));

            Ok(Some((key, parsed_value)))
        }

        fn parse_key(&mut self) -> Result<String> {
            if !self
                .line
                .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            {
                return Err(self.err());
            }
            let index = match self
                .line
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
            {
                Some(index) => index,
                None => self.line.len(),
            };
            self.pos += index;
            let key = String::from(&self.line[..index]);
            self.line = &self.line[index..];
            Ok(key)
        }

        fn expect_equal(&mut self) -> Result<()> {
            if !self.line.starts_with('=') {
                return Err(self.err());
            }
            self.line = &self.line[1..];
            self.pos += 1;
            Ok(())
        }

        fn skip_whitespace(&mut self) {
            if let Some(index) = self.line.find(|c: char| !c.is_whitespace()) {
                self.pos += index;
                self.line = &self.line[index..];
            } else {
                self.pos += self.line.len();
                self.line = "";
            }
        }
    }

    #[derive(Eq, PartialEq)]
    enum SubstitutionMode {
        None,
        Block,
        EscapedBlock,
    }

    #[allow(clippy::too_many_lines)]
    fn parse_value(
        input: &str,
        substitution_data: &mut BTreeMap<String, Option<String>>,
    ) -> Result<String> {
        let mut strong_quote = false;
        let mut weak_quote = false;
        let mut escaped = false;
        let mut expecting_end = false;

        let mut output = String::new();

        let mut substitution_mode = SubstitutionMode::None;
        let mut substitution_name = String::new();

        for (index, c) in input.chars().enumerate() {
            if expecting_end {
                if c == ' ' || c == '\t' {
                    continue;
                } else if c == '#' {
                    break;
                }
                return Err(Error::LineParse(input.to_owned(), index));
            } else if escaped {
                match c {
                    '\\' | '\'' | '"' | '$' | ' ' => output.push(c),
                    'n' => output.push('\n'),
                    _ => {
                        return Err(Error::LineParse(input.to_owned(), index));
                    }
                }

                escaped = false;
            } else if strong_quote {
                if c == '\'' {
                    strong_quote = false;
                } else {
                    output.push(c);
                }
            } else if substitution_mode != SubstitutionMode::None {
                if c.is_alphanumeric() {
                    substitution_name.push(c);
                } else {
                    match substitution_mode {
                        SubstitutionMode::None => unreachable!(),
                        SubstitutionMode::Block => {
                            if c == '{' && substitution_name.is_empty() {
                                substitution_mode = SubstitutionMode::EscapedBlock;
                            } else {
                                apply_substitution(
                                    substitution_data,
                                    &substitution_name.drain(..).collect::<String>(),
                                    &mut output,
                                );
                                if c == '$' {
                                    substitution_mode = if !strong_quote && !escaped {
                                        SubstitutionMode::Block
                                    } else {
                                        SubstitutionMode::None
                                    }
                                } else {
                                    substitution_mode = SubstitutionMode::None;
                                    output.push(c);
                                }
                            }
                        }
                        SubstitutionMode::EscapedBlock => {
                            if c == '}' {
                                substitution_mode = SubstitutionMode::None;
                                apply_substitution(
                                    substitution_data,
                                    &substitution_name.drain(..).collect::<String>(),
                                    &mut output,
                                );
                            } else {
                                substitution_name.push(c);
                            }
                        }
                    }
                }
            } else if c == '$' {
                substitution_mode = if !strong_quote && !escaped {
                    SubstitutionMode::Block
                } else {
                    SubstitutionMode::None
                }
            } else if weak_quote {
                if c == '"' {
                    weak_quote = false;
                } else if c == '\\' {
                    escaped = true;
                } else {
                    output.push(c);
                }
            } else if c == '\'' {
                strong_quote = true;
            } else if c == '"' {
                weak_quote = true;
            } else if c == '\\' {
                escaped = true;
            } else if c == ' ' || c == '\t' {
                expecting_end = true;
            } else {
                output.push(c);
            }
        }

        if substitution_mode == SubstitutionMode::EscapedBlock || strong_quote || weak_quote {
            let value_length = input.len();
            Err(Error::LineParse(
                input.to_owned(),
                if value_length == 0 {
                    0
                } else {
                    value_length - 1
                },
            ))
        } else {
            apply_substitution(
                substitution_data,
                &substitution_name.drain(..).collect::<String>(),
                &mut output,
            );
            Ok(output)
        }
    }

    fn apply_substitution(
        substitution_data: &mut BTreeMap<String, Option<String>>,
        substitution_name: &str,
        output: &mut String,
    ) {
        if let Ok(environment_value) = std::env::var(substitution_name) {
            output.push_str(&environment_value);
        } else {
            let stored_value = substitution_data
                .get(substitution_name)
                .unwrap_or(&None)
                .clone();
            output.push_str(&stored_value.unwrap_or_default());
        };
    }
}
