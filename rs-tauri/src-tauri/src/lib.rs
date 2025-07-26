use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager as _;

mod types;
mod winapi;

type State<'r> = tauri::State<'r, Mutex<types::AppState>>;

fn get_or_create_app_dir() -> Result<PathBuf, io::Error> {
    let dir = winapi::get_app_data_path().join("poe-hacks");
    match fs::create_dir(&dir) {
        Ok(_) => Ok(dir),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(dir),
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn get_profiles(state: State<'_>) -> Vec<types::ProfileDefinition> {
    state.lock().unwrap().profiles.clone()
}

#[tauri::command]
fn set_profiles(state: State<'_>, profiles: Vec<types::ProfileDefinition>) -> bool {
    let app_dir = match get_or_create_app_dir() {
        Ok(d) => d,
        Err(_) => return false,
    };

    let mut state = state.lock().unwrap();
    state.profiles = profiles;

    match fs::File::create(app_dir.join("poe-hacks.json")) {
        Ok(f) => serde_json::to_writer(io::BufWriter::new(f), &*state).is_ok(),
        Err(_) => return false,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = get_or_create_app_dir()?;
            let app_state = match fs::File::open(app_dir.join("poe-hacks.json")) {
                Ok(file) => {
                    serde_json::from_reader::<_, types::AppState>(io::BufReader::new(file))?
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => types::AppState {
                    profiles: vec![types::ProfileDefinition {
                        name: String::from("default"),
                        active: true,
                        rules: Vec::new(),
                    }],
                },
                Err(e) => return Err(e.into()),
            };
            app.manage(Mutex::new(app_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_profiles, set_profiles])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
