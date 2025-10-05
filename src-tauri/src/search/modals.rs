use rfd::AsyncFileDialog;

/// Opens a file dialog for selecting a single image file.
/// Accepts common image formats (PNG, JPEG, WEBP, etc.)
#[tauri::command]
pub async fn upload_image_file() -> Result<String, String> {
    if let Some(file) = AsyncFileDialog::new()
        .set_title("Select an Image File")
        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
        .pick_file()
        .await
    {
        Ok(file.path().to_string_lossy().to_string())
    } else {
        Err("No file selected".into())
    }
}

/// Opens a file dialog for selecting a single audio file.
/// Accepts common audio formats (MP3, WAV, FLAC, etc.)
#[tauri::command]
pub async fn upload_audio_file() -> Result<String, String> {
    if let Some(file) = AsyncFileDialog::new()
        .set_title("Select an Audio File")
        .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a"])
        .pick_file()
        .await
    {
        Ok(file.path().to_string_lossy().to_string())
    } else {
        Err("No file selected".into())
    }
}

/// Opens a file dialog for selecting a single document file.
/// Accepts text, PDF, and common office formats.
#[tauri::command]
pub async fn upload_document_file() -> Result<String, String> {
    if let Some(file) = AsyncFileDialog::new()
        .set_title("Select a Document File")
        .add_filter("Documents", &["txt", "pdf", "docx", "md", "rtf"])
        .pick_file()
        .await
    {
        Ok(file.path().to_string_lossy().to_string())
    } else {
        Err("No file selected".into())
    }
}
