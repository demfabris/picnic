<p align="center">
  <a href="https://github.com/demfabris/picnic">
    <img alt="picnic"
         width="350" height="280"
         src="https://github.com/demfabris/picnic/assets/46208058/ae98fcce-30f1-4548-8f9b-166bdccf58e5">
  </a>
</p>

<h2 align="center">
  PICNIC Is Config Notation Interpreter/Converter 🦀
</h2>

<p align="center">
  <a href="https://github.com/demfabris/picnic">
    <img alt="github" src="https://img.shields.io/badge/github-demfabris/picnic-8da0cb?style=for-the-badge&labelColor=555555&logo=github">
  </a>
  <a href="https://crates.io/crates/picnic-rs">
    <img src="https://img.shields.io/crates/v/picnic-rs?style=for-the-badge"/>
  </a>
</p>


# PICNIC

PICNIC's name is powered by AI, which immediately makes it worth your time:

> Human: Please come up with a recursive name for my cli project which interprets configuration files and prints environment variables.

> ChatGPT: That sounds like a useful tool! How about naming it PICNIC, which stands for "PICNIC Is Config Notation Interpreter/Converter". This fits the recursive acronym style you're looking for, and it also gives a sense of ease and simplicity, as if dealing with various config file formats is just a "picnic" with this tool.

## Features

✅ Extract data from `json` and `.env` files (soon `yaml`, `toml`, `csv`, `xml`) <br>
✅ Match keys with the same syntax as your file format <br>
✅ Output matched results, or everything. Source it with `eval` <br>
✅ Optionally spawn tiny binaries that print your values (useful when outside shell scripting, e.g. Nix)

## Installation

#### From [crates.io](https://crates.io/crates/picnic-rs)
`cargo install picnic-rs` (I'm trying to get ownership for `picnic`)

## Usage

Some json examples.
The usage is similar for other formats. `picnic --help` for more info.

```json
// some.json
{
  "foo": "bar",
  "baz": {
    "quz": "qork"
  },
  "boo": [
    "bah",
    {
      "lol": "lurg"
    }
  ]
}
```

#### `$ picnic some.json`

Output:
```sh
baz.quz=qork; export baz.quz;
boo.0=bah; export boo.0;
boo.1.lol=lurg; export boo.1.lol;
foo=bar; export foo;
```

Eval the output to set the environment variables:

```sh 
eval $(picnic some.json)
```

### ⭐ Matching templates

Replace the values you want to extract with `$` variables:

#### `$ picnic some.json --match '{"boo": [$BAH, "lol": $LURG] }'`

Output:
```sh
BAH=bah; export BAH;
LURG=lurg; export LURG;
```

Similarly, eval the output to set the env variables.

### 📝 Custom separators and casing options

#### `$ picnic some.json --separator _ --casing upper`

Output:
```sh
BAZ_QUZ=qork; export BAZ_QUZ;
BOO_0=bah; export BOO_0;
BOO_1_LOL=lurg; export BOO_1_LOL;
FOO=bar; export FOO;
```

### 💾 Spawn binaries

#### `$ picnic some.json --spawn /tmp`

Generates:
```sh
$ ls /tmp
foo
baz.quz
boo.0
boo.1.lol
```

Outputs:
```sh
$ ./foo
bar

$ ./baz.quz
qork

$ ./boo.0
bah

$ ./boo.1.lol
lurg
```

### ↩︎ Pipe stdin to picnic

```sh
curl -o some.json http://config.com/some_json_i_know_not_to_be_malicious.json 
eval $(cat some.json | picnic)
```

## Contributing
Contributions are welcome! Feel free to open an issue or submit a PR.

## License
APACHE-2.0 and MIT

## Disclaimer
Do not eval output or generate binaries from unknown files!