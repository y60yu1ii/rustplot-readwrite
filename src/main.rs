mod font_loader; // 引入字型模組

use dirs::config_dir;
use eframe::egui;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf; // ✅ 載入 `rfd` 檔案選擇器

#[derive(Debug, Serialize, Deserialize)]
struct DataConfig {
    data: f32,
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
    fn save(&self, path: Option<PathBuf>) {
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
    fn load(path: Option<PathBuf>) -> Self {
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

struct MyApp {
    config: DataConfig,
}

impl MyApp {
    fn new(ctx: &egui::Context) -> Self {
        font_loader::load_custom_font(ctx);
        Self {
            config: DataConfig::load(None), // 預設載入內建 `data.yaml`
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("輸入數值:");
            ui.add(egui::DragValue::new(&mut self.config.data));

            if ui.button("儲存到 YAML").clicked() {
                self.config.save(None);
            }

            if ui.button("從 YAML 載入").clicked() {
                self.config = DataConfig::load(None);
            }

            ui.separator();

            if ui.button("從外部檔案選擇 YAML").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    println!("📂 選擇 YAML 檔案: {:?}", path);
                    self.config = DataConfig::load(Some(path));
                    self.config.save(None);
                }
            }

            if ui.button("選擇儲存 YAML 位置").clicked() {
                if let Some(path) = FileDialog::new()
                    .set_file_name("data.yaml") // ✅ 預設檔名
                    .save_file()
                {
                    println!("💾 儲存 YAML 到: {:?}", path);
                    self.config.save(Some(path));
                }
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "YAML 儲存範例",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(&cc.egui_ctx)))), // ✅ Wrap inside Ok()
    )?;

    Ok(())
}
