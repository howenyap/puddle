use mime::Mime;
use std::ffi::OsStr;
use std::path::Path;

pub(crate) fn read_upload_file(
    path: &Path,
) -> Result<(Vec<u8>, Mime, String), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let file_name = file_name_string(path)?;

    Ok((bytes, mime, file_name))
}

pub(crate) fn file_name_string(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let file_name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("path has no file name: {}", path.display()),
        )
    })?;

    Ok(file_name.to_string())
}

pub(crate) fn display_text<'a>(value: Option<&'a str>, fallback: &'a str) -> &'a str {
    value
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(fallback)
}
