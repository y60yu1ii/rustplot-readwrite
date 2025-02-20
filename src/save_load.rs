use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataConfig {
    pub data: f32,
}

impl DataConfig {
    /// **取得 `data.yaml` 存放的位置**
    fn get_config_path() -> Option<PathBuf> {
        let mut path = config_dir()?; // 取得 config 資料夾 (`AppData` / `Library/Application Support`)
        path.push("egui-app"); // 應用程式名稱
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("⚠️ 無法建立設定資料夾: {:?}，使用臨時資料夾！", e);
            return None; // 改用當前目錄
        }
        path.push("data.yaml");
        Some(path)
    }

    /// **儲存 YAML**
    pub fn save(&self, path: Option<PathBuf>) {
        let path = path.unwrap_or_else(|| {
            Self::get_config_path().unwrap_or_else(|| PathBuf::from("data.yaml"))
        });

        match serde_yaml::to_string(self) {
            Ok(yaml) => {
                if let Err(e) = fs::write(&path, yaml) {
                    eprintln!(
                        "⚠️ 無法寫入 YAML 檔案: {:?}，請確認應用程式是否有寫入權限！",
                        e
                    );
                } else {
                    println!("✅ YAML 已儲存至 {:?}", path);
                }
            }
            Err(e) => {
                eprintln!("⚠️ YAML 轉換失敗: {:?}", e);
            }
        }
    }

    /// **載入 YAML**
    pub fn load(path: Option<PathBuf>) -> Self {
        let path = path.unwrap_or_else(|| {
            Self::get_config_path().unwrap_or_else(|| PathBuf::from("data.yaml"))
        });

        match fs::read_to_string(&path) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|_| {
                eprintln!("⚠️ YAML 格式錯誤，使用預設值！");
                Self { data: 3.0 }
            }),
            Err(e) => {
                eprintln!("⚠️ 找不到 `data.yaml`（錯誤: {:?}），建立新檔案！", e);
                Self { data: 3.0 }
            }
        }
    }
}
