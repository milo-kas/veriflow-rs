use serde::{Deserialize, Serialize};
pub mod hashing;
pub mod protocol;
use thiserror::Error;

// cli command arg
// PartialEQ for unit test
/// Primary header for the Veriflow protocol
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "command", content = "data")]
pub enum FileHeader {
    /// Upload file
    Upload {
        name: String,
        size: u64,    // u64 is standard for files
        hash: String, // hex string
    },

    /// Download file
    Download { name: String },

    /// Delete file
    Delete { name: String },

    /// Lists the directories from server's resource folder
    List, // No data required

    /// Server response to given request
    /// Success
    Success(String),

    /// Failure, something went wrong server-side
    Error(String),
}

// FileHeader Server Response Logic
impl FileHeader {
    /// Check if the server to client header is a Success or an Error then handle it
    pub fn unpack_response(self) -> Result<()> {
        match self {
            FileHeader::Success(msg) => {
                println!("Server: {msg}");
                Ok(())
            }
            FileHeader::Error(e) => Err(VeriflowError::ServerError(e)),
            other => Err(VeriflowError::UnexpectedFileHeader(format!("{:?}", other))),
        }
    }

    /// Helper to get the variant filename
    pub fn path(&self) -> &str {
        match self {
            FileHeader::Upload { name, .. } => name,
            FileHeader::Download { name } => name,
            FileHeader::Delete { name } => name,
            _ => "", // Other enums return empty string
        }
    }
}

// Error Type Struct for wrapping errors
#[derive(Error, Debug)]
pub enum VeriflowError {
    /// String Error
    #[error("String Conversion Error: {0}")]
    String(#[from] std::string::ParseError),
    /// IO Error
    #[error("Network/Disk Error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection Error
    #[error("Could not connect to {ip}. Details: {source}")]
    ConnectionFailed {
        ip: String,
        #[source]
        source: std::io::Error,
    },

    /// JSON Error
    #[error("Serialisation Error: {0}")]
    JSON(#[from] serde_json::Error),

    /// File Path Error
    #[error("Invalid Path: Could not extract a valid filename from the provided path")]
    InvalidPath,

    /// Hash Mismatch Error
    #[error("Hash Mismatch: The downloaded file was corrupted")]
    HashMismatch,

    /// Giant Header Error
    #[error("Security Alert: Requested header size {0} bytes exceeds the limit.")]
    HeaderSizeExceeded(usize),

    /// Giant Payload Error
    #[error("Security Alert: Requested payload size {0} bytes exceeds the limit.")]
    PayloadSizeExceeded(usize),

    /// Unexpected File Header Error
    #[error("Unexpected FileHeader: Received \"{0}\"")]
    UnexpectedFileHeader(String),

    /// Specific error message sent from server to client
    #[error("Server Error: {0}")]
    ServerError(String),

    /// TOML Error
    #[error("Serialisation Error: {0}")]
    TOMLser(#[from] toml::ser::Error),
    #[error("Deserialisation Error: {0}")]
    TOMLder(#[from] toml::de::Error),
}

// Allow writing Result<String> instead of Result<String, VeriflowError>
pub type Result<T> = std::result::Result<T, VeriflowError>;

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    // Test Serialisation and Deserialisation
    #[test]
    fn test_file_header_serialisation() {
        // set file name
        let file_name: &str = "img.png";

        // instantiate file header
        let original_file_header = FileHeader::Upload {
            name: String::from(file_name),
            size: 4001,
            hash: String::from("abc123def"),
        };
        // serialise to JSON (Struct -> String)
        let json_string_wrapped = serde_json::to_string(&original_file_header);
        // unwrap JSON
        let json_string = json_string_wrapped.unwrap();

        // test if file name is inside of json
        assert!(json_string.contains(file_name));
        assert!(json_string.contains("Upload"));

        // Deserialise (String -> Struct)
        let deserialised_json_wrapped = serde_json::from_str(&json_string);
        let deserialised_json = deserialised_json_wrapped.unwrap();

        assert_eq!(original_file_header, deserialised_json);
    }
    // Test VeriFlow error type struct
    #[test]
    fn test_error_conversion() {
        // parse non-json into FileHeader
        fn json_fail() -> super::Result<FileHeader> {
            let garbage = "not json";
            // 'from_str' tries to convert string to JSON,
            // the '?' operator handles the failure by automatically converting the JSON error to custom wrapper error
            let header: FileHeader = serde_json::from_str(garbage)?;
            Ok(header)
        }

        let result = json_fail();

        // test if resturned type is an Error
        assert!(result.is_err());

        // Verify the error type
        println!("{}", result.unwrap_err());
    }
}
