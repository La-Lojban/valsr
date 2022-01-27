# Valsr

![Valsr](/static/valsr.jpg)

A Lojban version of the word guessing game [Wordle](https://www.powerlanguage.co.uk/wordle/) implemented in [Rust](https://www.rust-lang.org).

Live version running at [la-lojban.github.io/valsr/](https://la-lojban.github.io/valsr/).

A fork of [sanuli](https://github.com/Cadiac/sanuli)

## Quick start

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

To build the WASM based [yew](https://yew.rs/) UI, further wasm tooling is required

```
$ rustup target add wasm32-unknown-unknown
$ cargo install --locked trunk
$ cargo install wasm-bindgen-cli
```

Create word list files and populate them with uppercase words, one per line

```
$ touch common-words.txt
$ touch daily-words.txt
$ touch full-words.txt
$ touch profanities.txt
```

Start the UI in development mode
```
$ RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --port=9090
```

## Word lists

Three separate word list files in the root of this project containing all the words are required. The lists are not included in this repository.

The lists are:
- `full-words.txt` - Full list of all accepted 5 and 6 character words. The checks if a word real or not is done against this list
- `daily-words.txt` - List of daily words. The daily word is taken from row equal to the days from 2022-01-07.
- `common-words.txt` - Subset of the full words list, intended for easier game mode. Note that all these words _must_ exist on the `full-words.txt`
- `profanities.txt` - Words filtered out when profanities filter is enabled

Beware that these are _included in the release binary_, and anyone can obtain the lists!

## Generating base word lists

To create a word list, a dictionary like the "nykysuomen sanalista" by [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/), licensed with [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/deed.fi), can be used as a baseline.

A parser for parsing `kotus-sanalista_v1.xml` file from [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/) is included:

```bash
$ cargo run --bin parse-kotus-word-list your/path/to/kotus-sanalista_v1.xml
```

which creates a `full-words-generated.txt` file in the working directory.

## Development

For development, start the web server with

```
$ RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve
```

This should make the UI available at 0.0.0.0:8080 with hot reload on code changes.

To change the default port, use

```
$ RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve --port=9090
```

## Release build

```
$ RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release
```

and copy the produced `docs` directory to your target server.
