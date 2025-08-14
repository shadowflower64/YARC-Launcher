use minisign::PError;
use serde::{Serialize, Serializer};
use std::{fmt, io, path::PathBuf};
use zip_extract::ZipExtractError;

#[derive(Serialize)]
pub enum CommandError {
    CreateYARCDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    CreateLauncherDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    CreateTempDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    CreateYARGDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    CreateSetlistDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    GetBaseDirs,
    ExtractSetlistPath {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: sevenz_rust::Error,
    },
    ExtractFileOpenError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    }, //
    ExtractZipError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: ZipExtractError,
    },
    UnhandledReleaseFileType(String),
    #[serde(serialize_with = "serialize_error_to_string")]
    InvalidSignatureFile(PError),
    #[serde(serialize_with = "serialize_error_to_string")]
    VerifyOpenZipFail(io::Error),
    #[serde(serialize_with = "serialize_error_to_string")]
    VerifyFail(PError),
    DownloadInitFail {
        url: String,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: reqwest::Error,
    },
    #[serde(serialize_with = "serialize_error_to_string")]
    DownloadFail(reqwest::Error),
    AnyOtherError(String),
}

pub fn serialize_error_to_string<E: fmt::Debug, S: Serializer>(
    error: &E,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{error:?}"))
}

impl CommandError {
    pub fn msg(&self) -> String {
        match self {
            Self::CreateYARCDirectory { .. } => "Failed to create YARC directory.",
            Self::CreateLauncherDirectory { .. } => "Failed to create launcher directory.",
            Self::CreateTempDirectory { .. } => "Failed to create launcher directory.",
            Self::CreateYARGDirectory { .. } => "Failed to create YARG directory.",
            Self::CreateSetlistDirectory { .. } => "Failed to create setlist directory.",
            Self::GetBaseDirs => "Failed to get base directories.",
            Self::ExtractSetlistPath { .. } => "Failed to extract setlist part.",
            Self::ExtractFileOpenError { .. } => "Failed to open file while extracting.",
            Self::ExtractZipError { .. } => "Failed to extract zip.",
            Self::UnhandledReleaseFileType(_) => "Unhandled release file type.",
            Self::InvalidSignatureFile(_) => "Invalid signature file! Try reinstalling. If it keeps failing, let us know ASAP!",
            Self::VerifyOpenZipFail(_) => "Failed to open zip while verifying.",
            Self::VerifyFail(_) => "Failed to verify downloaded zip file! Try reinstalling. If it keeps failing, let us know ASAP!",
            Self::DownloadInitFail { .. } => "Failed to initialize download.",
            Self::DownloadFail(_) => "Error while downloading file.",
            Self::AnyOtherError(msg) => msg,
            // _ => "Unknown error."
        }.to_owned()
    }
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::AnyOtherError(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::AnyOtherError(value.to_owned())
    }
}
