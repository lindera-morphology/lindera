use crate::error::LinderaErrorKind;
use crate::tokenizer::Token;
use crate::LinderaResult;

#[derive(Debug, Clone, Copy)]
/// Formatter type
pub enum Format {
    Mecab,
    Wakati,
    Json,
}

fn format_mecab(tokens: Vec<Token>) -> LinderaResult<String> {
    let mut lines = Vec::new();
    for token in tokens {
        let line = format!("{}\t{}", token.text, token.detail.join(","));
        lines.push(line);
    }
    lines.push(String::from("EOS"));

    Ok(lines.join("\n"))
}

fn format_wakati(tokens: Vec<Token>) -> LinderaResult<String> {
    let mut lines = Vec::new();
    for token in tokens {
        let line = token.text.to_string();
        lines.push(line);
    }

    Ok(lines.join(" "))
}

fn format_json(tokens: Vec<Token>) -> LinderaResult<String> {
    serde_json::to_string_pretty(&tokens)
        .map_err(|err| LinderaErrorKind::Serialize.with_error(anyhow::anyhow!(err)))
}

/// Format Token list to String by `output_format`
///
/// # Arguments
///
/// * `tokens`: the list of Token
/// * `output_format`: the mode of `Format`
///
/// returns: Result<String, LinderaError>
///
/// * String: formatted tokens
/// * LinderaError: the error occurred during formatting
///
pub fn format(tokens: Vec<Token>, output_format: Format) -> LinderaResult<String> {
    match output_format {
        Format::Mecab => format_mecab(tokens),
        Format::Wakati => format_wakati(tokens),
        Format::Json => format_json(tokens),
    }
}
