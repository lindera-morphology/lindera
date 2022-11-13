use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use lindera_core::character_filter::{add_offset_diff, CharacterFilter};

use crate::{error::LinderaErrorKind, LinderaResult};

pub const UNICODE_NORMALIZE_CHARACTER_FILTER_NAME: &str = "unicode_normalize";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum UnicodeNormalizeKind {
    #[serde(rename = "nfc")]
    NFC,
    #[serde(rename = "nfd")]
    NFD,
    #[serde(rename = "nfkc")]
    NFKC,
    #[serde(rename = "nfkd")]
    NFKD,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UnicodeNormalizeCharacterFilterConfig {
    pub kind: UnicodeNormalizeKind,
}

impl UnicodeNormalizeCharacterFilterConfig {
    pub fn new(kind: UnicodeNormalizeKind) -> Self {
        Self { kind }
    }

    pub fn from_slice(data: &[u8]) -> LinderaResult<Self> {
        serde_json::from_slice(data).map_err(|err| LinderaErrorKind::Deserialize.with_error(err))
    }
}

#[derive(Clone, Debug)]
pub struct UnicodeNormalizeCharacterFilter {
    config: UnicodeNormalizeCharacterFilterConfig,
}

impl UnicodeNormalizeCharacterFilter {
    pub fn new(config: UnicodeNormalizeCharacterFilterConfig) -> Self {
        Self { config }
    }

    pub fn from_slice(data: &[u8]) -> LinderaResult<Self> {
        Ok(Self::new(
            UnicodeNormalizeCharacterFilterConfig::from_slice(data)?,
        ))
    }
}

impl CharacterFilter for UnicodeNormalizeCharacterFilter {
    fn apply(&self, text: &mut String) -> LinderaResult<(Vec<usize>, Vec<i64>)> {
        let mut offsets: Vec<usize> = Vec::new();
        let mut diffs: Vec<i64> = Vec::new();

        let mut input_offset = 0;
        let mut replacement_offset = 0;

        let normalized_text = match self.config.kind {
            UnicodeNormalizeKind::NFC => text.nfc().collect::<String>(),
            UnicodeNormalizeKind::NFD => text.nfd().collect::<String>(),
            UnicodeNormalizeKind::NFKC => text.nfkc().collect::<String>(),
            UnicodeNormalizeKind::NFKD => text.nfkd().collect::<String>(),
        };

        let chars = text.chars();
        let mut normalized_chars = normalized_text.chars();

        // loop over the characters in the string
        for c in chars {
            let prefix_len = c.len_utf8();

            // To compare with the replaced character,
            // the character before normalization is retrieved and normalized.
            let tmp_text = match self.config.kind {
                UnicodeNormalizeKind::NFC => c.nfc().collect::<String>(),
                UnicodeNormalizeKind::NFD => c.nfd().collect::<String>(),
                UnicodeNormalizeKind::NFKC => c.nfkc().collect::<String>(),
                UnicodeNormalizeKind::NFKD => c.nfkd().collect::<String>(),
            };

            // Find a replacement from the normalized string that matches `tmp_c`.
            let mut replacement = String::new();
            let mut normalized_prefix_len = 0;
            for normalized_c in normalized_chars.by_ref() {
                normalized_prefix_len += normalized_c.len_utf8();
                replacement = normalized_text
                    [replacement_offset..replacement_offset + normalized_prefix_len]
                    .to_string();
                if replacement == tmp_text {
                    replacement_offset += normalized_prefix_len;
                    break;
                }
            }

            let replacement_len = replacement.len();
            let diff = prefix_len as i64 - replacement_len as i64;
            input_offset += prefix_len;

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
        }

        *text = normalized_text;

        Ok((offsets, diffs))
    }
}

#[cfg(test)]
mod tests {
    use lindera_core::character_filter::CharacterFilter;

    use crate::character_filter::unicode_normalize::{
        UnicodeNormalizeCharacterFilter, UnicodeNormalizeCharacterFilterConfig,
    };

    #[test]
    fn test_unicode_normalize_character_filter_config_from_slice() {
        let config_str = r#"
        {
            "kind": "nfkc"
        }
        "#;
        let config =
            UnicodeNormalizeCharacterFilterConfig::from_slice(config_str.as_bytes()).unwrap();

        assert_eq!(config.kind, super::UnicodeNormalizeKind::NFKC);
    }

    #[test]
    fn test_unicode_normalize_character_filter_from_slice() {
        let config_str = r#"
        {
            "kind": "nfkc"
        }
        "#;
        let result = UnicodeNormalizeCharacterFilter::from_slice(config_str.as_bytes());

        assert_eq!(true, result.is_ok());
    }

    #[test]
    fn test_unicode_normalize_character_filter_apply_nfc() {
        let config_str = r#"
        {
            "kind": "nfc"
        }
        "#;
        let filter = UnicodeNormalizeCharacterFilter::from_slice(config_str.as_bytes()).unwrap();

        let mut text = "ＡＢＣＤＥ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ＡＢＣＤＥ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ABCDE".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ABCDE", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ｱｲｳｴｵ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ｱｲｳｴｵ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "アイウエオ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("アイウエオ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "０１２３４５６７８９".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("０１２３４５６７８９", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ﾘﾝﾃﾞﾗ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ﾘﾝﾃﾞﾗ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "１０㌎".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("１０㌎", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);
    }

    #[test]
    fn test_unicode_normalize_character_filter_apply_nfd() {
        let config_str = r#"
        {
            "kind": "nfd"
        }
        "#;
        let filter = UnicodeNormalizeCharacterFilter::from_slice(config_str.as_bytes()).unwrap();

        let mut text = "ＡＢＣＤＥ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ＡＢＣＤＥ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ABCDE".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ABCDE", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ｱｲｳｴｵ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ｱｲｳｴｵ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "アイウエオ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("アイウエオ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "０１２３４５６７８９".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("０１２３４５６７８９", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ﾘﾝﾃﾞﾗ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ﾘﾝﾃﾞﾗ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "１０㌎".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("１０㌎", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);
    }

    #[test]
    fn test_unicode_normalize_character_filter_apply_nfkc() {
        let config_str = r#"
        {
            "kind": "nfkc"
        }
        "#;
        let filter = UnicodeNormalizeCharacterFilter::from_slice(config_str.as_bytes()).unwrap();

        let mut text = "ＡＢＣＤＥ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ABCDE", text);
        assert_eq!(vec![1, 2, 3, 4, 5], offsets);
        assert_eq!(vec![2, 4, 6, 8, 10], diffs);

        let mut text = "ABCDE".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ABCDE", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ｱｲｳｴｵ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("アイウエオ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "アイウエオ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("アイウエオ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "０１２３４５６７８９".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("0123456789", text);
        assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], offsets);
        assert_eq!(vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20], diffs);

        let mut text = "ﾘﾝﾃﾞﾗ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("リンデラ", text);
        assert_eq!(vec![9, 10, 11, 12], offsets);
        assert_eq!(vec![-1, -2, -3, 3], diffs);

        let mut text = "１０㌎".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("10ガロン", text);
        assert_eq!(vec![1, 2, 5, 6, 7, 8, 9, 10], offsets);
        assert_eq!(vec![2, 4, 3, 2, 1, 0, -1, -2], diffs);
    }

    #[test]
    fn test_unicode_normalize_character_filter_apply_nfkd() {
        let config_str = r#"
        {
            "kind": "nfkd"
        }
        "#;
        let filter = UnicodeNormalizeCharacterFilter::from_slice(config_str.as_bytes()).unwrap();

        let mut text = "ＡＢＣＤＥ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ABCDE", text);
        assert_eq!(vec![1, 2, 3, 4, 5], offsets);
        assert_eq!(vec![2, 4, 6, 8, 10], diffs);

        let mut text = "ABCDE".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("ABCDE", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "ｱｲｳｴｵ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("アイウエオ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "アイウエオ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("アイウエオ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "０１２３４５６７８９".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("0123456789", text);
        assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], offsets);
        assert_eq!(vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20], diffs);

        let mut text = "ﾘﾝﾃﾞﾗ".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("リンテ\u{3099}ラ", text);
        assert_eq!(Vec::<usize>::new(), offsets);
        assert_eq!(Vec::<i64>::new(), diffs);

        let mut text = "１０㌎".to_string();
        let (offsets, diffs) = filter.apply(&mut text).unwrap();
        assert_eq!("10カ\u{3099}ロン", text);
        assert_eq!(vec![1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13], offsets);
        assert_eq!(vec![2, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5], diffs);
    }
}
