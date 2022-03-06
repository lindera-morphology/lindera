# Lindera

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Join the chat at https://gitter.im/lindera-morphology/lindera](https://badges.gitter.im/lindera-morphology/lindera.svg)](https://gitter.im/lindera-morphology/lindera?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

A morphological analysis library in Rust. This project fork from fulmicoton's [kuromoji-rs](https://github.com/fulmicoton/kuromoji-rs).

Lindera aims to build a library which is easy to install and provides concise APIs for various Rust applications.

## Build

The following products are required to build:

- Rust >= 1.46.0

```text
% cargo build --release
```

### Build with IPADIC

The "ipadic" feature flag allows Lindera to include IPADIC. 

```shell script
% cargo build --release --features=ipadic
```

### Build with UniDic

The "unidic" feature flag allows Lindera to include UniDic. 

```shell script
% cargo build --release --features=unidic
```

### Build small binary

You can reduce the size of the binary containing the lindera by using the "compress" feature flag.  
Instead, you will be penalized for the execution time of the program.

This repo example is this.

```sh
% cargo build --release --features compress
```

It also depends on liblzma to compress the dictionary. Please install the dependent packages as follows:

```text
% sudo apt install liblzma-dev
```

## Usage

### Basic example

This example covers the basic usage of Lindera.

It will:
- Create a tokenizer in normal mode
- Tokenize the input text
- Output the tokens

```rust
use lindera::tokenizer::Tokenizer;
use lindera_core::LinderaResult;

fn main() -> LinderaResult<()> {
    // create tokenizer
    let tokenizer = Tokenizer::new()?;

    // tokenize the text
    let tokens = tokenizer.tokenize("関西国際空港限定トートバッグ")?;

    // output the tokens
    for token in tokens {
        println!("{}", token.text);
    }

    Ok(())
}
```

The above example can be run as follows:

```shell script
% cargo run --features=ipadic --example=basic_example
```

You can see the result as follows:
```text
関西国際空港
限定
トートバッグ
```

### User dictionary example

You can give user dictionary entries along with the default system dictionary. User dictionary should be a CSV with following format.

```
<surface_form>,<part_of_speech>,<reading>
```

For example:
```shell
% cat userdic.csv
東京スカイツリー,カスタム名詞,トウキョウスカイツリー
東武スカイツリーライン,カスタム名詞,トウブスカイツリーライン
とうきょうスカイツリー駅,カスタム名詞,トウキョウスカイツリーエキ
```

With an user dictionary, `Tokenizer` will be created as follows:
```rust
use std::path::PathBuf;

use lindera::tokenizer::{Tokenizer, TokenizerConfig};
use lindera_core::viterbi::Mode;
use lindera_core::LinderaResult;

fn main() -> LinderaResult<()> {
    // create tokenizer
    let config = TokenizerConfig {
        user_dict_path: Some(PathBuf::from("./resources/userdic.csv")),
        mode: Mode::Normal,
        ..TokenizerConfig::default()
    };
    let tokenizer = Tokenizer::with_config(config)?;

    // tokenize the text
    let tokens = tokenizer.tokenize("東京スカイツリーの最寄り駅はとうきょうスカイツリー駅です")?;

    // output the tokens
    for token in tokens {
        println!("{}", token.text);
    }

    Ok(())
}
```

The above example can be by `cargo run --example`:
```shell
% cargo run --features=ipadic --example=userdic_example
東京スカイツリー
の
最寄り駅
は
とうきょうスカイツリー駅
です
```

## API reference

The API reference is available. Please see following URL:
- <a href="https://docs.rs/lindera" target="_blank">lindera</a>
