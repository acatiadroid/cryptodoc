use std::io;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum FileError {
    DialogClosed,
    IOFailed(io::ErrorKind),
}

pub fn get_file_path(file_name: &String) -> PathBuf {
    PathBuf::from(format!(
        "{}\\documents\\{}.cryptodoc",
        env!("CARGO_MANIFEST_DIR"),
        file_name
    ))
}

pub fn pathbuf_to_string(path: &PathBuf) -> String {
    path.to_str()
        .expect("Failed to convert path to str")
        .to_string()
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), FileError> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(FileError::IOFailed)?;

    Ok((path, contents))
}

pub async fn pick_file() -> Result<(PathBuf, Arc<String>), FileError> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Select file")
        .pick_file()
        .await
        .ok_or(FileError::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

pub async fn pick_folder() -> Result<PathBuf, FileError> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Select folder")
        .pick_folder()
        .await
        .ok_or(FileError::DialogClosed)?;

    Ok(handle.path().to_owned())
}

pub async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, FileError> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file")
            .save_file()
            .await
            .ok_or(FileError::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, text)
        .await
        .map_err(|error| FileError::IOFailed(error.kind()))?;

    Ok(path)
}
