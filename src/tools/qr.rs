use std::io::Cursor;
use std::time::Duration;

use url::Url;
use qrcode::QrCode;
use serde_json::{json, Value};
use image::{Luma, ImageFormat, load_from_memory};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::blocking::{Client, multipart::Form, multipart::Part};

use super::consts::*;
use crate::server::ToolHandler;

#[derive(Debug)]
enum QrToolName {
    Generate,
    Decode,
}

impl QrToolName {
    fn as_str(&self) -> &'static str {
        match self {
            QrToolName::Generate => "generate_qr_code",
            QrToolName::Decode => "decode_qr_code",
        }
    }
}


pub struct QrGeneratorTool;

impl ToolHandler for QrGeneratorTool {
    fn name(&self) -> &str {
        QrToolName::Generate.as_str()
    }

    fn description(&self) -> &str {
        "Generate QR code from text or URL input"
    }

    fn input_schema(&self) -> Value {
        qr_generator_schema()
    }

    fn execute(&self, params: Option<Value>) -> Result<Value, String> {
        let mut content = extract_string_param(&params, "content")?;
        
        if content.len() > MAX_CONTENT_LENGTH {
            return Err(ERR_CONTENT_TOO_LONG.to_string());
        }
        
        if content.starts_with("http://") {
            content = content.replace("http://", "https://");
        }
        
        let qr_code = QrCode::new(&content)
            .map_err(|_| ERR_QR_CREATION_FAILED.to_string())?;

        let image = qr_code.render::<Luma<u8>>()
            .min_dimensions(DEFAULT_QR_SIZE as u32, DEFAULT_QR_SIZE as u32)
            .build();

        let mut buffer = Vec::new();
        image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .map_err(|_| ERR_IMAGE_ENCODING_FAILED.to_string())?;

        let upload_url = match upload_image(&buffer) {
            Ok(url) => url,
            Err(_) => return Err(ERR_UPLOAD_FAILED.to_string())
        };
        
        Ok(json!({
            "url": upload_url
        }))
    }
}

pub struct QrDecoderTool;

impl ToolHandler for QrDecoderTool {
    fn name(&self) -> &str {
        QrToolName::Decode.as_str()
    }

    fn description(&self) -> &str {
        "Decode QR code from base64 image data or HTTPS image URL (trusted domains only)"
    }

    fn input_schema(&self) -> Value {
        qr_decoder_schema()
    }

    fn execute(&self, params: Option<Value>) -> Result<Value, String> {
        let mut image_data = extract_string_param(&params, "image_data")?;
        
        if image_data.starts_with("http://") { // cuz like we need https, just forcing https seems to be fine
            image_data = image_data.replace("http://", "https://");
        }
        
        let image_bytes = if image_data.starts_with("http") {
            download_image(&image_data)?
        } else {
            if image_data.len() > MAX_BASE64_SIZE {
                return Err(ERR_BASE64_TOO_LARGE.to_string());
            }
            let decoded = STANDARD.decode(strip_data_url_prefix(&image_data))
                .map_err(|_| ERR_BASE64_DECODE_FAILED.to_string())?;
            if decoded.len() > MAX_FILE_SIZE {
                return Err(ERR_INVALID_FILE_SIZE.to_string());
            }
            decoded
        };
        
        let img = load_from_memory(&image_bytes)
            .map_err(|_| ERR_IMAGE_LOAD_FAILED.to_string())?;
        
        if img.width() > MAX_IMAGE_DIMENSION || img.height() > MAX_IMAGE_DIMENSION {
            return Err(ERR_IMAGE_TOO_LARGE.to_string());
        }
        
        let gray_img = img.to_luma8();
        let mut decoder = quircs::Quirc::new();
        let codes = decoder.identify(img.width() as usize, img.height() as usize, &gray_img);

        for code in codes.flatten() {
            if let Ok(decoded) = code.decode() {
                return Ok(json!({
                    "content": String::from_utf8_lossy(&decoded.payload)
                }));
            }
        }
        
        Err(ERR_NO_QR_CODE_FOUND.to_string())
    }
}

fn extract_string_param(params: &Option<Value>, key: &str) -> Result<String, String> {
    params
        .as_ref()
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}


fn strip_data_url_prefix(data: &str) -> &str {
    if let Some(comma_pos) = data.find(',') {
        &data[comma_pos + 1..]
    } else {
        data
    }
}

fn validate_url(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|_| ERR_INVALID_URL.to_string())?;
    
    if parsed.scheme() != "https" {
        return Err(ERR_HTTPS_REQUIRED.to_string());
    }
    
    let host = parsed.host_str().ok_or(ERR_INVALID_DOMAIN.to_string())?;
    if !ALLOWED_DOMAINS.iter().any(|&d| host == d || host.ends_with(&format!(".{}", d))) {
        return Err(ERR_DOMAIN_NOT_ALLOWED.to_string());
    }
    
    let path = parsed.path().to_lowercase();
    if !IMAGE_EXTENSIONS.iter().any(|&ext| path.ends_with(ext)) {
        return Err(ERR_MUST_BE_IMAGE_FILE.to_string());
    }
    
    Ok(())
}


fn download_image(url: &str) -> Result<Vec<u8>, String> {
    validate_url(url)?;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .user_agent("qrmax/1.0")
        .build()
        .map_err(|_| ERR_HTTP_CLIENT_ERROR.to_string())?;
    
    let response = client.get(url).send()
        .map_err(|_| ERR_DOWNLOAD_FAILED.to_string())?;
    
    if !response.status().is_success() {
        return Err(ERR_HTTP_ERROR.to_string());
    }
    
    if let Some(ct) = response.headers().get("content-type") {
        if !ct.to_str().unwrap_or("").starts_with("image/") {
            return Err(ERR_NOT_AN_IMAGE.to_string());
        }
    }
    
    if let Some(len) = response.headers().get("content-length") {
        if let Ok(size) = len.to_str().unwrap_or("").parse::<usize>() {
            if size > MAX_FILE_SIZE {
                return Err(ERR_TOO_LARGE.to_string());
            }
        }
    }
    
    let data = response.bytes().map_err(|_| ERR_READ_FAILED.to_string())?.to_vec();
    if data.len() > MAX_FILE_SIZE {
        return Err(ERR_INVALID_FILE_SIZE.to_string());
    }
    Ok(data)
}

fn upload_image(image_data: &[u8]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if image_data.len() > MAX_FILE_SIZE {
        return Err(ERR_INVALID_FILE_SIZE.into());
    }
    
    let client = Client::builder()
        .timeout(Duration::from_secs(UPLOAD_TIMEOUT_SECS))
        .user_agent("qrmax/1.0")
        .build()?;
    
    let form = Form::new()
        .part("reqtype", Part::text("fileupload"))
        .part("fileToUpload", Part::bytes(image_data.to_vec())
            .file_name("qr.png")
            .mime_str("image/png")?);
    
    let resp = client.post("https://catbox.moe/user/api.php")
        .multipart(form)
        .send()?;
    
    let url = resp.text()?.trim().to_string();
    
    if url.starts_with("https://") && url.contains("catbox.moe") {
        Ok(url)
    } else {
        Err(ERR_UPLOAD_FAILED.into())
    }
}

