use log::warn;
use minisign::PError;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::{error::Error, fmt, io, path::PathBuf};
use zip_extract::ZipExtractError;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum CommandError {
    ConvertPathToStringError(PathBuf),
    FailedToRecreateFolder {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
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
    },
    ExtractZipError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: ZipExtractError,
    },
    UnhandledReleaseFileType(String),
    WriteTagFileError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    #[serde(serialize_with = "serialize_error_to_string")]
    InvalidSignatureFile(PError),
    #[serde(serialize_with = "serialize_error_to_string")]
    VerifyOpenZipFail(io::Error),
    #[serde(serialize_with = "serialize_error_to_string")]
    VerifyFail(PError),
    DownloadFileCreateFail {
        path: PathBuf,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    DownloadInitFail {
        url: String,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: reqwest::Error,
    },
    DownloadWriteError {
        path: PathBuf,
        url: String,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    },
    #[serde(serialize_with = "serialize_error_to_string")]
    DownloadFail(reqwest::Error),
    FailedToRemoveTagFile {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    FailedToLaunchProfile {
        path: PathBuf,
        arguments: Vec<String>,
        use_obs_vkcapture: bool,
        #[serde(serialize_with = "serialize_error_to_string")]
        error: io::Error,
    }
    // AnyOtherError(String),
}

pub fn serialize_io_error<S: Serializer>(
    error: &io::Error,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut error_info = serializer.serialize_struct("error_info", 5)?;
    error_info.serialize_field("kind", &error.kind().to_string())?;
    error_info.serialize_field("raw", &error.raw_os_error())?;
    error_info.serialize_field("string", &error.to_string())?;
    error_info.serialize_field("debug_format", &format!("{error:?}"))?;
    error_info.end()
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
            Self::ConvertPathToStringError(_) => "Failed to convert path to string!",
            Self::FailedToRecreateFolder { .. } => "Failed to re-create folder.",
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
            Self::WriteTagFileError { .. } => "Failed to write tag file.",
            Self::InvalidSignatureFile(_) => "Invalid signature file! Try reinstalling. If it keeps failing, let us know ASAP!",
            Self::VerifyOpenZipFail(_) => "Failed to open zip while verifying.",
            Self::VerifyFail(_) => "Failed to verify downloaded zip file! Try reinstalling. If it keeps failing, let us know ASAP!",
            Self::DownloadFileCreateFail { .. } => "Failed to create download file.",
            Self::DownloadInitFail { .. } => "Failed to initialize download.",
            Self::DownloadWriteError { .. } => "Error while writing to file.",
            Self::DownloadFail(_) => "Error while downloading file.",
            Self::FailedToRemoveTagFile { .. } => "Failed to remove tag file.",
            Self::FailedToLaunchProfile { use_obs_vkcapture: false, .. } => "Failed to launch profile! Is the executable installed?",
            Self::FailedToLaunchProfile { use_obs_vkcapture: true, .. } => "Failed to launch profile! Is the executable installed? Is obs-vkcapture installed and pathed?",
            // Self::AnyOtherError(msg) => msg,
            // _ => "Unknown error."
        }.to_owned()
    }
}

// impl From<String> for CommandError {
//     fn from(value: String) -> Self {
//         Self::AnyOtherError(value)
//     }
// }

// impl From<&str> for CommandError {
//     fn from(value: &str) -> Self {
//         Self::AnyOtherError(value.to_owned())
//     }
// }
