use std::path::PathBuf;

async fn save_file(path: Option<PathBuf>, text:String) -> Result<PathBuf> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name")
            .save_file()
            .await
            .ok()
    };
}