#[repr(C)]
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Health {
    pub hp: i32,
    pub max_hp: i32,
    pub unreserved_hp: i32,
    pub es: i32,
    pub max_es: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Mana {
    pub mana: i32,
    pub max_mana: i32,
    pub unreserved_mana: i32,
}

#[tauri::command]
fn greet(name: &str) -> Health {
    Health {
        hp: name.parse().unwrap_or(50),
        max_hp: 100,
        unreserved_hp: 100,
        es: 100,
        max_es: name.parse().unwrap_or(50),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
