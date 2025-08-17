use minisign::PError;
use opener::OpenError;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::{error::Error, io, path::PathBuf};
use zip_extract::ZipExtractError;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Err {
    ConvertPathToStringError {
        path: PathBuf
    },
    FailedToRecreateFolder {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    CreateYARCDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    CreateLauncherDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    CreateTempDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    CreateYARGDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    CreateSetlistDirectory {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    GetBaseDirs,
    ExtractSetlistPath {
        path: PathBuf,
        #[serde(serialize_with = "serialize_any_error")]
        error: sevenz_rust::Error,
    },
    ExtractFileOpenError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    ExtractZipError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_any_error")]
        error: ZipExtractError,
    },
    #[serde(rename_all = "camelCase")]
    UnhandledReleaseFileType {
        release_type: String
    },
    WriteTagFileError {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    InvalidSignatureFile {
        #[serde(serialize_with = "serialize_any_error")]
        error: PError
    },
    VerifyOpenZipFail {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error
    },
    VerifyFail {
        #[serde(serialize_with = "serialize_any_error")]
        error: PError
    },
    DownloadFileCreateFail {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    DownloadInitFail {
        url: String,
        #[serde(serialize_with = "serialize_any_error")]
        error: reqwest::Error,
    },
    DownloadWriteError {
        path: PathBuf,
        url: String,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    DownloadFail {
        #[serde(serialize_with = "serialize_any_error")]
        error: reqwest::Error
    },
    FailedToRemoveTagFile {
        path: PathBuf,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    #[serde(rename_all = "camelCase")]
    FailedToLaunchProfile {
        path: PathBuf,
        arguments: Vec<String>,
        use_obs_vkcapture: bool,
        #[serde(serialize_with = "serialize_io_error")]
        error: io::Error,
    },
    FailedToRevealFolder {
        path: PathBuf,
        #[serde(serialize_with = "serialize_any_error")]
        error: OpenError,
    }, // AnyOtherError(String),
}

pub fn serialize_io_error<S: Serializer>(
    error: &io::Error,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut error_info = serializer.serialize_struct("ErrorInfo", 4)?;
    error_info.serialize_field("kind", &error.kind().to_string())?;
    error_info.serialize_field("rawOsError", &error.raw_os_error())?;
    error_info.serialize_field("description", &error.to_string())?;
    error_info.serialize_field("debugFormat", &format!("{error:?}"))?;
    error_info.end()
}

pub fn serialize_any_error<E: Error, S: Serializer>(
    error: &E,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut error_info = serializer.serialize_struct("ErrorInfo", 2)?;
    error_info.serialize_field("description", &error.to_string())?;
    error_info.serialize_field("debugFormat", &format!("{error:?}"))?;
    error_info.end()
}

impl Err {
    pub fn msg(&self) -> String {
        match self {
            Self::ConvertPathToStringError{..} => "Failed to convert path to string!",
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
            Self::UnhandledReleaseFileType { .. } => "Unhandled release file type.",
            Self::WriteTagFileError { .. } => "Failed to write tag file.",
            Self::InvalidSignatureFile{ .. } => "Invalid signature file! Try reinstalling. If it keeps failing, let us know ASAP!",
            Self::VerifyOpenZipFail{..} => "Failed to open zip while verifying.",
            Self::VerifyFail{..} => "Failed to verify downloaded zip file! Try reinstalling. If it keeps failing, let us know ASAP!",
            Self::DownloadFileCreateFail { .. } => "Failed to create download file.",
            Self::DownloadInitFail { .. } => "Failed to initialize download.",
            Self::DownloadWriteError { .. } => "Error while writing to file.",
            Self::DownloadFail { .. } => "Error while downloading file.",
            Self::FailedToRemoveTagFile { .. } => "Failed to remove tag file.",
            Self::FailedToLaunchProfile { use_obs_vkcapture: false, .. } => "Failed to launch profile! Is the executable installed?",
            Self::FailedToLaunchProfile { use_obs_vkcapture: true, .. } => "Failed to launch profile! Is the executable installed? Is obs-vkcapture installed and pathed?",
            Self::FailedToRevealFolder { .. } => "Failed to reveal folder. Is it installed?",
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

#[derive(Debug)]
pub struct CommandError(Err);

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut error_info = serializer.serialize_struct("CommandError", 2)?;
        error_info.serialize_field("details", &self.0)?;
        error_info.serialize_field("message", &self.0.msg())?;
        error_info.end()
    }
}

impl From<Err> for CommandError {
    fn from(value: Err) -> Self {
        Self(value)
    }
}
