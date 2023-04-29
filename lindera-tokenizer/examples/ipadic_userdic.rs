use lindera_core::LinderaResult;

fn main() -> LinderaResult<()> {
    #[cfg(feature = "ipadic")]
    {
        use std::path::PathBuf;

        use lindera_core::mode::Mode;
        use lindera_dictionary::{DictionaryConfig, DictionaryKind, UserDictionaryConfig};
        use lindera_tokenizer::tokenizer::{Tokenizer, TokenizerConfig};

        let dictionary = DictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: None,
        };

        let user_dictionary = Some(UserDictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../resources")
                .join("ipadic_simple_userdic.csv"),
        });

        let config = TokenizerConfig {
            dictionary,
            user_dictionary: user_dictionary,
            mode: Mode::Normal,
        };

        #[allow(unused_variables)]
        let tokenizer = Tokenizer::from_config(config).unwrap();

        // tokenize the text
        let tokens =
            tokenizer.tokenize("東京スカイツリーの最寄り駅はとうきょうスカイツリー駅です")?;

        // output the tokens
        for token in tokens {
            println!("{}", token.text);
        }
    }

    Ok(())
}
