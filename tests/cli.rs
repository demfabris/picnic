use assert_cmd::{crate_name, Command};
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::env;

pub fn cmd() -> Command {
    Command::cargo_bin(crate_name!()).unwrap()
}

mod json {
    use super::*;

    const INVALID_JSON: &str = r#"{"key": value}"#;
    const VALID_JSON: &str = r#"{"key": "value"}"#;
    const VALID_COMPLEX_JSON: &str =
        r#" { "foo": "bar", "baz": { "quz": "qork" }, "boo": [ "bah", { "lol": "lurg" } ] } "#;

    // Test that the CLI fails when the input file is a invalid json
    #[test]
    fn test_invalid_json() {
        let mut cmd = cmd();
        let file = assert_fs::NamedTempFile::new("invalid.json").unwrap();
        file.write_str(INVALID_JSON).unwrap();
        cmd.arg(file.path());
        dbg!(cmd.output().unwrap());
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Json(Error(\"expected value\", line: 1, column: 9))\n",
        ));
    }

    #[test]
    fn test_valid_json() {
        let mut cmd = cmd();
        let file = assert_fs::NamedTempFile::new("valid.json").unwrap();
        file.write_str(VALID_JSON).unwrap();
        cmd.arg(file.path());
        dbg!(cmd.output().unwrap());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("key=value; export key;"));
    }

    #[test]
    fn test_valid_complex_json() {
        let mut cmd = cmd();
        let file = assert_fs::NamedTempFile::new("valid_complex.json").unwrap();
        file.write_str(VALID_COMPLEX_JSON).unwrap();
        cmd.arg(file.path());
        dbg!(cmd.output().unwrap());
        cmd.assert().success().stdout(predicate::str::contains(
            r#"baz.quz=qork; export baz.quz;
boo.0=bah; export boo.0;
boo.1.lol=lurg; export boo.1.lol;
foo=bar; export foo;"#,
        ));
    }

    #[test]
    fn test_valid_complex_json_with_matches() {
        let mut cmd = cmd();
        let file = assert_fs::NamedTempFile::new("valid_complex.json").unwrap();
        file.write_str(VALID_COMPLEX_JSON).unwrap();
        cmd.arg(file.path())
            .arg("--match")
            .arg(r#"{"foo": $BAR, "boo": [$BAH, { "lol": $LURG }] }"#);
        dbg!(cmd.output().unwrap());
        cmd.assert().success().stdout(predicate::str::contains(
            r#"BAH=bah; export BAH;
LURG=lurg; export LURG;
BAR=bar; export BAR;"#,
        ));
    }

    #[test]
    fn test_valid_complex_json_custom_separator() {
        let mut cmd = cmd();
        let file = assert_fs::NamedTempFile::new("valid_complex.json").unwrap();
        file.write_str(VALID_COMPLEX_JSON).unwrap();
        cmd.arg(file.path()).arg("--separator").arg("_");
        dbg!(cmd.output().unwrap());
        cmd.assert().success().stdout(predicate::str::contains(
            r#"baz_quz=qork; export baz_quz;
boo_0=bah; export boo_0;
boo_1_lol=lurg; export boo_1_lol;
foo=bar; export foo;"#,
        ));
    }

    #[test]
    fn test_valid_complex_json_custom_separator_and_upper_casing() {
        let mut cmd = cmd();
        let file = assert_fs::NamedTempFile::new("valid_complex.json").unwrap();
        file.write_str(VALID_COMPLEX_JSON).unwrap();
        cmd.arg(file.path())
            .arg("--separator")
            .arg("_")
            .arg("--casing")
            .arg("upper");
        dbg!(cmd.output().unwrap());
        cmd.assert().success().stdout(predicate::str::contains(
            r#"BAZ_QUZ=qork; export BAZ_QUZ;
BOO_0=bah; export BOO_0;
BOO_1_LOL=lurg; export BOO_1_LOL;
FOO=bar; export FOO;"#,
        ));
    }
}
