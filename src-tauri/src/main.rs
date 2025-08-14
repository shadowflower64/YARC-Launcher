// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod types;
mod utils;
mod command_error;

use std::{
    fs::{self, File}, path::PathBuf, process::Command, sync::{LazyLock, Mutex}
};

use clap::Parser;
use directories::BaseDirs;
use minisign::{PublicKeyBox, SignatureBox};
use tauri::{AppHandle, Emitter, Manager};
use types::*;
use utils::*;

use crate::command_error::{CommandError, PathCtx};

const YARG_PUB_KEY: &str = "untrusted comment: minisign public key C26EBBBEC4C1DB81
RWSB28HEvrtuwvPn3pweVBodgVi/d+UH22xDsL3K8VBgeRqaIrDdTvps
";

static COMMAND_LINE_ARG_LAUNCH: LazyLock<Mutex<Option<String>>> =
    LazyLock::new(|| Mutex::new(None));

#[tauri::command(async)]
fn get_important_dirs() -> Result<ImportantDirs, CommandError> {
    // Get the important directories
    let dirs = BaseDirs::new().ok_or(CommandError::GetBaseDirsError)?;
    let yarc_folder = PathBuf::from(dirs.data_local_dir()).join("YARC");
    let launcher_folder = PathBuf::from(&yarc_folder).join("Launcher");
    let temp_folder = PathBuf::from(&launcher_folder).join("Temp");

    // Create the directories if they don't exist
    fs::create_dir_all(&yarc_folder).map_err(|e| CommandError::new_create_dir_err(PathCtx::YARC, &yarc_folder, e))?;
    fs::create_dir_all(&launcher_folder).map_err(|e| CommandError::new_create_dir_err(PathCtx::Launcher, &launcher_folder, e))?;
    fs::create_dir_all(&temp_folder).map_err(|e| CommandError::new_create_dir_err(PathCtx::Temp, &temp_folder, e))?;

    Ok(ImportantDirs {
        yarc_folder: path_to_string(yarc_folder)?,
        launcher_folder: path_to_string(launcher_folder)?,
        temp_folder: path_to_string(temp_folder)?,
    })
}

#[tauri::command(async)]
fn get_custom_dirs(download_location: String) -> Result<CustomDirs, CommandError> {
    // Get the custom directories
    let yarg_folder = PathBuf::from(&download_location).join("YARG Installs");
    let setlist_folder = PathBuf::from(&download_location).join("Setlists");

    // Create the directories if they don't exist
    fs::create_dir_all(&yarg_folder).map_err(|e| CommandError::new_create_dir_err(PathCtx::YARG, &yarg_folder, e))?;
    fs::create_dir_all(&setlist_folder).map_err(|e| CommandError::new_create_dir_err(PathCtx::Setlist, &setlist_folder, e))?;

    Ok(CustomDirs {
        yarg_folder: path_to_string(yarg_folder)?,
        setlist_folder: path_to_string(setlist_folder)?,
    })
}

#[tauri::command]
fn is_dir_empty(path: String) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

#[tauri::command(async)]
fn is_connected_to_internet() -> bool {
    online::check(Some(7)).is_ok()
}

#[tauri::command(async)]
fn profile_folder_state(path: String, wanted_tag: String) -> ProfileFolderState {
    let tag_file = PathBuf::from(&path).join("tag.txt");

    let tag_file_exists = tag_file.try_exists();
    if let Ok(exists) = tag_file_exists {
        if !exists {
            return ProfileFolderState::FirstDownload;
        }

        let tag = fs::read_to_string(tag_file);
        if let Ok(tag_string) = tag {
            if tag_string.trim() == wanted_tag {
                return ProfileFolderState::UpToDate;
            } else {
                return ProfileFolderState::UpdateRequired;
            }
        } else {
            eprintln!("Failed to read tag file at `{}`", path);
            return ProfileFolderState::Error;
        }
    } else {
        eprintln!("Failed to find if the profile exists at `{}`", path);
        return ProfileFolderState::Error;
    }
}

// when i was getting disk space in rust i used "free_space" from the fs2 crate because it takes a path and works out what drive that would be

#[tauri::command(async)]
async fn download_and_install_profile(
    handle: AppHandle,
    profile_path: String,
    uuid: String,
    tag: String,
    temp_path: String,
    content: Vec<ReleaseContent>,
) -> Result<(), String> {
    let temp_file = PathBuf::from(&temp_path).join(format!("{}.temp", uuid));
    let _ = fs::remove_file(&temp_file);

    let sig_file = PathBuf::from(&temp_path).join(format!("{}.temp_sig", uuid));
    let _ = fs::remove_file(&sig_file);

    let install_path = PathBuf::from(&profile_path).join("installation");
    clear_folder(&install_path)?;

    // Download and install all content
    let current_os = std::env::consts::OS.to_string();
    for c in content {
        // Skip release content that is not for this OS
        if !c.platforms.iter().any(|i| i.to_owned() == current_os) {
            continue;
        }

        let file_count = c.files.len() as u64;
        for (index, file) in c.files.iter().enumerate() {
            // Download
            download(
                Some(&handle),
                &file.url,
                &temp_file,
                file_count,
                index as u64,
            )
            .await?;

            let payload_current = (index + 1) as u64;

            // Verify (if signature is provided)
            if let Some(sig_url) = &file.sig_url {
                // Emit the verification
                let _ = &handle.emit(
                    "progress_info",
                    ProgressPayload {
                        state: "verifying".to_string(),
                        current: payload_current,
                        total: file_count,
                    },
                );

                // Download sig file (don't pass app so it doesn't emit an update)
                download(None, &sig_url, &sig_file, 0, 0).await?;

                // Convert public key
                let pk_box = PublicKeyBox::from_string(YARG_PUB_KEY).unwrap();
                let pk = pk_box.into_public_key().unwrap();

                // Create the signature box
                let sig_box = SignatureBox::from_file(&sig_file)
                    .map_err(|e| format!("Invalid signature file! Try reinstalling. If it keeps failing, let us know ASAP!\n{:?}", e))?;

                // Verify
                let zip_file = File::open(&temp_file)
                    .map_err(|e| format!("Failed to open zip while verifying.\n{:?}", e))?;
                minisign::verify(&pk, &sig_box, zip_file, true, false, false)
                    .map_err(|_| "Failed to verify downloaded zip file! Try reinstalling. If it keeps failing, let us know ASAP!")?;
            }

            // Extract/install
            let _ = handle.emit(
                "progress_info",
                ProgressPayload {
                    state: "installing".to_string(),
                    current: payload_current,
                    total: file_count,
                },
            );

            if file.file_type == "zip" {
                extract(&temp_file, &install_path)?;
            } else if file.file_type == "encrypted" {
                extract_encrypted(&temp_file, &install_path)?;
            } else {
                return Err("Unhandled release file type.".to_string());
            }

            // Clean up
            let _ = fs::remove_file(&temp_file);
            let _ = fs::remove_file(&sig_file);
        }
    }

    let tag_file = PathBuf::from(&profile_path).join("tag.txt");
    fs::write(&tag_file, tag).map_err(|e| format!("Failed to write tag file.\n{:?}", e))?;

    Ok(())
}

#[tauri::command(async)]
fn uninstall_profile(profile_path: String) -> Result<(), String> {
    let install_path = PathBuf::from(&profile_path).join("installation");
    clear_folder(&install_path)?;

    let tag_file = PathBuf::from(&profile_path).join("tag.txt");
    fs::remove_file(tag_file).map_err(|e| format!("Failed to remove tag file.\n{:?}", e))?;

    // Remove the directories if they are empty
    let _ = fs::remove_dir(&install_path);
    let _ = fs::remove_dir(&profile_path);

    Ok(())
}

#[tauri::command]
fn launch_profile(
    profile_path: String,
    exec_path: String,
    use_obs_vkcapture: bool,
    arguments: Vec<String>,
) -> Result<(), String> {
    let path = PathBuf::from(&profile_path).join("installation").join(exec_path);

    if !use_obs_vkcapture {
        Command::new(path).args(arguments).spawn().map_err(|e| {
            format!(
                "Failed to launch profile! Is the executable installed?\n{:?}",
                e
            )
        })?;
    } else {
        let path_str = path_to_string(path)?;

        Command::new("obs-gamecapture")
            .args([path_str].iter().chain(&arguments))
            .spawn()
            .map_err(|e| format!("Failed to launch profile! Is the executable installed? Is obs-vkcapture installed and pathed?\n{:?}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn open_folder_profile(profile_path: String) -> Result<(), String> {
    let path = PathBuf::from(&profile_path).join("installation");

    opener::reveal(path)
        .map_err(|e| format!("Failed to reveal folder. Is it installed?\n{:?}", e))?;

    Ok(())
}

#[tauri::command(async)]
fn get_launch_argument() -> Option<String> {
    let launch_arg = COMMAND_LINE_ARG_LAUNCH.lock().unwrap();
    return launch_arg.to_owned();
}

#[tauri::command(async)]
fn clean_up_old_install(yarg_folder: String, setlist_folder: String) -> Result<(), String> {
    let stable_old = PathBuf::from(&yarg_folder).join("stable");
    let _ = fs::remove_dir_all(&stable_old).inspect_err(|e| eprintln!("Failed to remove old stable directory: {e:?}"));

    let nightly_old = PathBuf::from(&yarg_folder).join("nightly");
    let _ = fs::remove_dir_all(&nightly_old).inspect_err(|e| eprintln!("Failed to remove old nightly directory: {e:?}"));

    let setlist_old = PathBuf::from(&setlist_folder).join("official");
    let _ = fs::remove_dir_all(&setlist_old).inspect_err(|e| eprintln!("Failed to remove old setlist directory: {e:?}"));

    Ok(())
}

fn main() {
    let args = CommandLineArgs::parse();

    {
        // Stores the launch option in a static so the frontend can request it later.
        let mut launch_option = COMMAND_LINE_ARG_LAUNCH.lock().unwrap();
        *launch_option = args.launch;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            get_important_dirs,
            get_custom_dirs,
            is_dir_empty,
            is_connected_to_internet,
            profile_folder_state,
            download_and_install_profile,
            uninstall_profile,
            launch_profile,
            open_folder_profile,
            get_launch_argument,
            clean_up_old_install
        ])
        .setup(|app| {
            // Show the window's shadow
            app.get_webview_window("main")
                .unwrap()
                .set_shadow(true)
                .unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application.");
}
