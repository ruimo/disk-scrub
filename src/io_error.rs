use std::{path::{PathBuf}, io::Error};

#[derive(Debug)]
pub struct IoError {
    pub path: Option<PathBuf>,
    pub message: String,
    pub cause: Error,
}