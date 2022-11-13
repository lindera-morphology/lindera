use regex::Regex;
use serde::{Deserialize, Serialize};

use lindera_core::character_filter::{add_offset_diff, CharacterFilter};

use crate::{error::LinderaErrorKind, LinderaResult};

pub const REGEX_CHARACTER_FILTER_NAME: &str = "regex";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RegexCharacterFilterConfig {
    pub pattern: String,
    pub replacement: String,
}

impl RegexCharacterFilterConfig {
    pub fn new(pattern: String, replacement: String) -> Self {
        Self {
            pattern,
            replacement,
        }
    }

    pub fn from_slice(data: &[u8]) -> LinderaResult<Self> {
        serde_json::from_slice(data).map_err(|err| LinderaErrorKind::Deserialize.with_error(err))
    }
}

#[derive(Clone, Debug)]
pub struct RegexCharacterFilter {
    config: RegexCharacterFilterConfig,
    regex: Regex,
}

impl RegexCharacterFilter {
    pub fn new(config: RegexCharacterFilterConfig) -> LinderaResult<Self> {
        let regex =
            Regex::new(&config.pattern).map_err(|err| LinderaErrorKind::Args.with_error(err))?;

        Ok(Self { config, regex })
    }

    pub fn from_slice(data: &[u8]) -> LinderaResult<Self> {
        Self::new(RegexCharacterFilterConfig::from_slice(data)?)
    }
}

impl CharacterFilter for RegexCharacterFilter {
    fn apply(&self, text: &mut String) -> LinderaResult<(Vec<usize>, Vec<i64>)> {
        let mut offsets: Vec<usize> = Vec::new();
        let mut diffs: Vec<i64> = Vec::new();

        self.regex.find_iter(text).for_each(|mat| {
            let target = mat.as_str();
            let target_len = target.len();
            let replacement = self.config.replacement.as_str();
            let replacement_len = replacement.len();
            let diff = target_len as i64 - replacement_len as i64;
            let input_offset = mat.start() + target_len;

            if diff != 0 {
                let prev_diff = *diffs.last().unwrap_or(&0);

                if diff > 0 {
                    // Replacement is shorter than matched surface.
                    add_offset_diff(
                        &mut offsets,
                        &mut diffs,
                        (input_offset as i64 - diff - prev_diff) as usize,
                        prev_diff + diff,
                    );
                } else {
                    // Replacement is longer than matched surface.
                    let output_start = (input_offset as i64 + -prev_diff) as usize;
                    for extra_idx in 0..diff.unsigned_abs() as usize {
                        add_offset_diff(
                            &mut offsets,
                            &mut diffs,
                            output_start + extra_idx,
                            prev_diff - extra_idx as i64 - 1,
                        );
                    }
                }
            }
        });

        *text = self
            .regex
            .replace_all(text, &self.config.replacement)
            .to_mut()
            .to_string();

        Ok((offsets, diffs))
    }
}

#[cfg(test)]
mod tests {
    use lindera_core::character_filter::{correct_offset, CharacterFilter};

    use crate::character_filter::regex::{RegexCharacterFilter, RegexCharacterFilterConfig};

    #[test]
    fn test_regex_character_filter_config_from_slice() {
        let config_str = r#"
        {
            "pattern": "リンデラ",
            "replacement": "Lindera"
        }
        "#;
        let config = RegexCharacterFilterConfig::from_slice(config_str.as_bytes()).unwrap();
        assert_eq!("リンデラ", config.pattern);
        assert_eq!("Lindera", config.replacement);
    }

    #[test]
    fn test_regex_character_filter_from_slice() {
        let config_str = r#"
        {
            "pattern": "リンデラ",
            "replacement": "Lindera"
        }
        "#;
        let result = RegexCharacterFilterConfig::from_slice(config_str.as_bytes());
        assert_eq!(true, result.is_ok());
    }

    #[test]
    fn test_regex_character_filter_apply() {
        {
            let config_str = r#"
            {
                "pattern": "リンデラ",
                "replacement": "Lindera"
            }
            "#;
            let filter = RegexCharacterFilter::from_slice(config_str.as_bytes()).unwrap();
            let text = "リンデラは形態素解析器です。".to_string();
            let mut filterd_text = text.clone();
            let (offsets, diffs) = filter.apply(&mut filterd_text).unwrap();
            assert_eq!("Linderaは形態素解析器です。", filterd_text);
            assert_eq!(vec![7], offsets);
            assert_eq!(vec![5], diffs);
            let start = 0;
            let end = 7;
            assert_eq!("Lindera", &filterd_text[start..end]);
            let correct_start = correct_offset(start, &offsets, &diffs, filterd_text.len());
            let correct_end = correct_offset(end, &offsets, &diffs, filterd_text.len());
            assert_eq!(0, correct_start);
            assert_eq!(12, correct_end);
            assert_eq!("リンデラ", &text[correct_start..correct_end]);
        }

        {
            let config_str = r#"
            {
                "pattern": "\\s{2,}",
                "replacement": " "
            }
            "#;
            let filter = RegexCharacterFilter::from_slice(config_str.as_bytes()).unwrap();
            let text = "a     b     c".to_string();
            let mut filterd_text = text.clone();
            let (offsets, diffs) = filter.apply(&mut filterd_text).unwrap();
            assert_eq!("a b c", filterd_text);
            assert_eq!(vec![2, 4], offsets);
            assert_eq!(vec![4, 8], diffs);
            let start = 2;
            let end = 3;
            assert_eq!("b", &filterd_text[start..end]);
            let correct_start = correct_offset(start, &offsets, &diffs, filterd_text.len());
            let correct_end = correct_offset(end, &offsets, &diffs, filterd_text.len());
            assert_eq!(6, correct_start);
            assert_eq!(7, correct_end);
            assert_eq!("b", &text[correct_start..correct_end]);
        }
    }
}
