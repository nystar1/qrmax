use serde_json::{json, Value};

pub const DEFAULT_QR_SIZE: i64 = 100;
pub const MAX_CONTENT_LENGTH: usize = 2048;
pub const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
pub const MAX_IMAGE_DIMENSION: u32 = 2048;
pub const DOWNLOAD_TIMEOUT_SECS: u64 = 10;
pub const UPLOAD_TIMEOUT_SECS: u64 = 30;
pub const MAX_BASE64_SIZE: usize = 10 * 1024 * 1024;

pub const ALLOWED_DOMAINS: &[&str] = &[
    "catbox.moe",
    "files.catbox.moe"
]; // thanks rowan for introducing me to catbox lol

pub const IMAGE_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".gif", ".webp"];

pub const ERR_CONTENT_TOO_LONG: &str = "Content too long";
pub const ERR_QR_CREATION_FAILED: &str = "QR creation failed";
pub const ERR_IMAGE_ENCODING_FAILED: &str = "Image encoding failed";
pub const ERR_UPLOAD_FAILED: &str = "Upload failed";
pub const ERR_BASE64_TOO_LARGE: &str = "Base64 too large";
pub const ERR_BASE64_DECODE_FAILED: &str = "Base64 decode failed";
pub const ERR_IMAGE_LOAD_FAILED: &str = "Image load failed";
pub const ERR_IMAGE_TOO_LARGE: &str = "Image too large";
pub const ERR_NO_QR_CODE_FOUND: &str = "No QR code found";
pub const ERR_INVALID_URL: &str = "Invalid URL";
pub const ERR_HTTPS_REQUIRED: &str = "HTTPS required";
pub const ERR_INVALID_DOMAIN: &str = "Invalid domain";
pub const ERR_DOMAIN_NOT_ALLOWED: &str = "Domain not allowed";
pub const ERR_MUST_BE_IMAGE_FILE: &str = "Must be image file";
pub const ERR_INVALID_FILE_SIZE: &str = "Invalid file size";
pub const ERR_HTTP_CLIENT_ERROR: &str = "HTTP client error";
pub const ERR_DOWNLOAD_FAILED: &str = "Download failed";
pub const ERR_HTTP_ERROR: &str = "HTTP error";
pub const ERR_NOT_AN_IMAGE: &str = "Not an image";
pub const ERR_TOO_LARGE: &str = "Too large";
pub const ERR_READ_FAILED: &str = "Read failed";

pub fn qr_generator_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "content": {
                "type": "string",
                "description": "Text to encode in QR code"
            }
        },
        "required": ["content"]
    })
}

pub fn qr_decoder_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "image_data": {
                "type": "string",
                "description": "Base64 encoded image data (PNG/JPEG) or HTTPS URL to an image from trusted domains (catbox.moe, files.catbox.moe)"
            }
        },
        "required": ["image_data"]
    })
}