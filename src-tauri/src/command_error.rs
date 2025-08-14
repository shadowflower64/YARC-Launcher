use std::{fmt, io, path::{Path, PathBuf}};
use serde::{Serialize, Serializer};


#[derive(Serialize)]
pub enum PathCtx {
    YARC,
    Launcher,
    Temp,
    YARG,
    Setlist
}

#[derive(Serialize)]
pub enum CommandError {
    CreateDirectoryError{
        context: PathCtx,
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error_to_string")] 
        error: io::Error,
    },
    GetBaseDirsError,
    UnknownStringError(String)
}

pub fn serialize_io_error_to_string<S>(error: &io::Error, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(&format!("{error:?}"))
}

impl CommandError {
    pub fn new_create_dir_err(context: PathCtx, path: &Path, error: io::Error) -> Self {
        Self::CreateDirectoryError { context, path: path.to_owned(), error }
    }

    pub fn new_unknown(msg: &str) -> Self {
        Self::UnknownStringError(msg.to_owned())
    }

    pub fn msg(&self) -> String {
        match self {
            Self::CreateDirectoryError{ context, .. } => match *context {
                PathCtx::YARC => "Failed to create YARC directory.",
                PathCtx::Launcher => "Failed to create launcher directory.",
                PathCtx::Temp => "Failed to create temp directory.",
                PathCtx::YARG => "Failed to create YARG directory.",
                PathCtx::Setlist => "Failed to create setlist directory."
            }.to_owned(),
            Self::GetBaseDirsError => "Failed to get base directories.".to_owned(),
            Self::UnknownStringError(msg) => msg.to_owned(),
            // _ => "Unknown error."
        }
    } 
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::UnknownStringError(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::UnknownStringError(value.to_owned())
    }
}