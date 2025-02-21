use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataConfig {
    pub data: f32,
}

impl DataConfig {
    pub fn get_config_path() -> Option<PathBuf> {
        let mut path = config_dir()?; // 取得 config 資料夾 (`AppData` / `Library/Application Support`)
        path.push("egui-app"); // 應用程式名稱
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("⚠️ 無法建立設定資料夾: {:?}，使用臨時資料夾！", e);
            return None;
        }
        path.push("data.yaml");
        Some(path)
    }
}
