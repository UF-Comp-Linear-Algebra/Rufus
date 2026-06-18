use base64::engine::general_purpose::{STANDARD as B64_STANDARD, URL_SAFE as B64_URL_SAFE};
use base64::Engine;
use clap::ValueEnum;

use crate::gradescope::types::Submitter;

#[derive(Debug, Clone)]
pub struct KeySpec {
    pub key: String,
    pub decode: Decode,
    pub ext: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decode {
    Raw,
    Base64,
    Base64Url,
    Hex,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Layout {
    Dir,
    Flat,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum NameBy {
    #[value(name = "submission_id")]
    SubmissionId,
    Name,
    Email,
    Sid,
}

/// Parses a `--key` flag value of the form `name[:decode[:ext]]`.
/// Any segment may be empty (e.g. `checksum::sha256` → raw decode, .sha256 ext).
pub fn parse_key_spec(s: &str) -> Result<KeySpec, String> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    let key = parts[0].to_string();
    if key.is_empty() {
        return Err("Key name cannot be empty".to_string());
    }
    let decode = match parts.get(1).copied().unwrap_or("") {
        "" | "none" | "raw" => Decode::Raw,
        "base64" => Decode::Base64,
        "base64url" => Decode::Base64Url,
        "hex" => Decode::Hex,
        other => {
            return Err(format!(
                "Unknown decode '{}'. Valid options: raw, none, base64, base64url, hex",
                other
            ))
        }
    };
    let ext = parts
        .get(2)
        .copied()
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Ok(KeySpec { key, decode, ext })
}

pub enum DecodeOutput {
    Text(String),
    Binary(Vec<u8>),
}

pub fn decode_value(value: &str, decode: &Decode) -> Result<DecodeOutput, String> {
    match decode {
        Decode::Raw => Ok(DecodeOutput::Text(value.to_string())),
        Decode::Base64 => {
            let bytes = B64_STANDARD
                .decode(value.trim())
                .map_err(|e| format!("base64 decode error: {}", e))?;
            Ok(DecodeOutput::Binary(bytes))
        }
        Decode::Base64Url => {
            let bytes = B64_URL_SAFE
                .decode(value.trim())
                .map_err(|e| format!("base64url decode error: {}", e))?;
            Ok(DecodeOutput::Binary(bytes))
        }
        Decode::Hex => {
            let s = value
                .trim()
                .trim_start_matches("0x")
                .trim_start_matches("0X");
            if s.len() % 2 != 0 {
                return Err("hex decode error: odd number of hex digits".to_string());
            }
            let bytes = (0..s.len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&s[i..i + 2], 16)
                        .map_err(|e| format!("hex decode error at offset {}: {}", i, e))
                })
                .collect::<Result<Vec<u8>, String>>()?;
            Ok(DecodeOutput::Binary(bytes))
        }
    }
}

/// Converts a serde_yaml Value to a String for writing or decoding.
/// Maps/sequences are serialized as trimmed YAML.
pub fn stringify_yaml_value(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Null => String::new(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

/// Replaces characters unsafe for use in file/directory names with `_`.
pub fn sanitize_for_path(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Computes the filesystem label for a submission based on `--name-by`.
/// Multi-submitter groups are joined with `_`. Falls back to `submission_id`
/// if the submitter list is empty.
pub fn compute_label(submission_id: &str, submitters: &[Submitter], name_by: &NameBy) -> String {
    let join = |parts: Vec<String>| {
        if parts.is_empty() {
            sanitize_for_path(submission_id)
        } else {
            parts.join("_")
        }
    };
    match name_by {
        NameBy::SubmissionId => sanitize_for_path(submission_id),
        NameBy::Name => join(
            submitters
                .iter()
                .map(|s| sanitize_for_path(&s.name))
                .collect(),
        ),
        NameBy::Email => join(
            submitters
                .iter()
                .map(|s| sanitize_for_path(&s.email))
                .collect(),
        ),
        NameBy::Sid => join(
            submitters
                .iter()
                .map(|s| sanitize_for_path(s.sid.as_deref().unwrap_or("unknown")))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sub(name: &str, email: &str, sid: Option<&str>) -> Submitter {
        Submitter {
            name: name.to_string(),
            email: email.to_string(),
            sid: sid.map(str::to_string),
        }
    }

    // --- parse_key_spec ---

    #[test]
    fn parse_key_only() {
        let k = parse_key_spec("level_uf2").unwrap();
        assert_eq!(k.key, "level_uf2");
        assert_eq!(k.decode, Decode::Raw);
        assert_eq!(k.ext, None);
    }

    #[test]
    fn parse_key_with_decode() {
        let k = parse_key_spec("level_uf2:base64").unwrap();
        assert_eq!(k.decode, Decode::Base64);
        assert_eq!(k.ext, None);
    }

    #[test]
    fn parse_key_full() {
        let k = parse_key_spec("level_uf2:base64:uf2").unwrap();
        assert_eq!(k.key, "level_uf2");
        assert_eq!(k.decode, Decode::Base64);
        assert_eq!(k.ext, Some("uf2".to_string()));
    }

    #[test]
    fn parse_key_empty_decode_with_ext() {
        // checksum::sha256 → raw decode, .sha256 ext
        let k = parse_key_spec("checksum::sha256").unwrap();
        assert_eq!(k.key, "checksum");
        assert_eq!(k.decode, Decode::Raw);
        assert_eq!(k.ext, Some("sha256".to_string()));
    }

    #[test]
    fn parse_key_none_decode_alias() {
        let k = parse_key_spec("foo:none:txt").unwrap();
        assert_eq!(k.decode, Decode::Raw);
    }

    #[test]
    fn parse_key_hex_decode() {
        let k = parse_key_spec("data:hex:bin").unwrap();
        assert_eq!(k.decode, Decode::Hex);
        assert_eq!(k.ext, Some("bin".to_string()));
    }

    #[test]
    fn parse_key_base64url() {
        let k = parse_key_spec("token:base64url").unwrap();
        assert_eq!(k.decode, Decode::Base64Url);
    }

    #[test]
    fn parse_key_empty_name_errors() {
        assert!(parse_key_spec(":base64:uf2").is_err());
    }

    #[test]
    fn parse_key_unknown_decode_errors() {
        assert!(parse_key_spec("key:magic").is_err());
    }

    // --- decode_value ---

    #[test]
    fn decode_raw_passthrough() {
        match decode_value("hello world", &Decode::Raw).unwrap() {
            DecodeOutput::Text(s) => assert_eq!(s, "hello world"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn decode_base64_bytes() {
        // "hello" → "aGVsbG8="
        match decode_value("aGVsbG8=", &Decode::Base64).unwrap() {
            DecodeOutput::Binary(b) => assert_eq!(b, b"hello"),
            _ => panic!("expected binary"),
        }
    }

    #[test]
    fn decode_base64_strips_whitespace() {
        match decode_value("  aGVsbG8=  ", &Decode::Base64).unwrap() {
            DecodeOutput::Binary(b) => assert_eq!(b, b"hello"),
            _ => panic!("expected binary"),
        }
    }

    #[test]
    fn decode_base64_invalid_errors() {
        assert!(decode_value("not!!valid", &Decode::Base64).is_err());
    }

    #[test]
    fn decode_hex_bytes() {
        // "hello" → "68656c6c6f"
        match decode_value("68656c6c6f", &Decode::Hex).unwrap() {
            DecodeOutput::Binary(b) => assert_eq!(b, b"hello"),
            _ => panic!("expected binary"),
        }
    }

    #[test]
    fn decode_hex_0x_prefix() {
        match decode_value("0x68656c6c6f", &Decode::Hex).unwrap() {
            DecodeOutput::Binary(b) => assert_eq!(b, b"hello"),
            _ => panic!("expected binary"),
        }
    }

    #[test]
    fn decode_hex_odd_length_errors() {
        assert!(decode_value("abc", &Decode::Hex).is_err());
    }

    #[test]
    fn decode_hex_invalid_chars_errors() {
        assert!(decode_value("zz", &Decode::Hex).is_err());
    }

    // --- stringify_yaml_value ---

    #[test]
    fn stringify_string_value() {
        let v = serde_yaml::Value::String("hello".to_string());
        assert_eq!(stringify_yaml_value(&v), "hello");
    }

    #[test]
    fn stringify_number_value() {
        let v = serde_yaml::Value::Number(serde_yaml::Number::from(42i64));
        assert_eq!(stringify_yaml_value(&v), "42");
    }

    #[test]
    fn stringify_bool_value() {
        assert_eq!(stringify_yaml_value(&serde_yaml::Value::Bool(true)), "true");
    }

    #[test]
    fn stringify_null_value() {
        assert_eq!(stringify_yaml_value(&serde_yaml::Value::Null), "");
    }

    // --- sanitize_for_path ---

    #[test]
    fn sanitize_spaces_to_underscores() {
        assert_eq!(sanitize_for_path("John Doe"), "John_Doe");
    }

    #[test]
    fn sanitize_email_at_sign() {
        assert_eq!(sanitize_for_path("jdoe@ufl.edu"), "jdoe_ufl.edu");
    }

    #[test]
    fn sanitize_alphanumeric_dash_dot_unchanged() {
        assert_eq!(sanitize_for_path("abc-123.bin"), "abc-123.bin");
    }

    // --- compute_label ---

    #[test]
    fn label_uses_submission_id() {
        let subs = vec![sub("John", "j@ufl.edu", Some("11111"))];
        assert_eq!(
            compute_label("99999", &subs, &NameBy::SubmissionId),
            "99999"
        );
    }

    #[test]
    fn label_by_name_single() {
        let subs = vec![sub("John Doe", "j@ufl.edu", Some("11111"))];
        assert_eq!(compute_label("99999", &subs, &NameBy::Name), "John_Doe");
    }

    #[test]
    fn label_by_name_multi_submitter() {
        let subs = vec![
            sub("John Doe", "j@ufl.edu", Some("11111")),
            sub("Jane Smith", "s@ufl.edu", Some("22222")),
        ];
        assert_eq!(
            compute_label("99999", &subs, &NameBy::Name),
            "John_Doe_Jane_Smith"
        );
    }

    #[test]
    fn label_by_email() {
        let subs = vec![sub("John", "jdoe@ufl.edu", None)];
        assert_eq!(
            compute_label("99999", &subs, &NameBy::Email),
            "jdoe_ufl.edu"
        );
    }

    #[test]
    fn label_by_sid_missing_falls_back_to_unknown() {
        let subs = vec![sub("John", "j@ufl.edu", None)];
        assert_eq!(compute_label("99999", &subs, &NameBy::Sid), "unknown");
    }

    #[test]
    fn label_empty_submitters_falls_back_to_submission_id() {
        assert_eq!(compute_label("99999", &[], &NameBy::Name), "99999");
    }
}
