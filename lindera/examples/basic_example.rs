use lindera::tokenizer::Tokenizer;
use lindera_core::LinderaResult;

fn main() -> LinderaResult<()> {
    // create tokenizer
    let mut tokenizer = Tokenizer::new()?;

    // tokenize the text
    let tokens = tokenizer.tokenize("関西国際空港限定トートバッグ")?;

    // output the tokens
    for token in tokens {
        println!("{}", token.text);
    }

    Ok(())
}
