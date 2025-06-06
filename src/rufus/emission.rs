use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;

#[derive(Clone)]
pub struct Emission {
    id: String,
    value: String,
}

pub enum EmissionParseError {
    FormatError(String),
    DecodeError(String),
}

impl std::fmt::Display for EmissionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmissionParseError::FormatError(msg) => write!(f, "Format Error: {}", msg),
            EmissionParseError::DecodeError(msg) => write!(f, "Decode Error: {}", msg),
        }
    }
}

impl Emission {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    fn decode_b64_str(encoded_string: &str) -> Result<String, String> {
        let decoded_bytes = b64.decode(encoded_string).map_err(|e| e.to_string())?;
        Ok(String::from_utf8(decoded_bytes).map_err(|e| e.to_string())?)
    }

    pub fn parse(emission_str: &str) -> Result<Emission, EmissionParseError> {
        let regex = regex::Regex::new(r"\*(.*?)\*(.*)").unwrap();

        let caps = regex
            .captures(emission_str)
            .ok_or_else(|| EmissionParseError::FormatError("Invalid format".to_string()))?;

        // We know there are 3 groups: the whole match, the id, and the encoded value
        let id = caps
            .get(1)
            .ok_or(EmissionParseError::FormatError(
                "ID not found in emission string".to_string(),
            ))? // use of the "Try operator" to return early if the ID is not found
            .as_str()
            .to_string();

        // Deserialize the bytes to get the value
        let value = Self::decode_b64_str(
            caps.get(2)
                .ok_or(EmissionParseError::FormatError(
                    "Encoded value not found in emission string".to_string(),
                ))?
                .as_str(),
        )
        .map_err(|e| EmissionParseError::DecodeError(e))?;

        Ok(Emission { id, value })
    }
}
