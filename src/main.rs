mod font_loader; // 引入字型模組

use dirs::config_dir;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
    fn save(&self) {
        if let Some(path) = Self::get_config_path() {
            match serde_yaml::to_string(self) {
                Ok(yaml) => {
                    if let Err(e) = fs::write(&path, yaml) {
                        eprintln!(
                            "⚠️ 無法寫入 YAML 檔案: {:?}，請確認應用程式是否有寫入權限！",
                            e
                        );
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ YAML 轉換失敗: {:?}", e);
                }
            }
        } else {
            eprintln!("⚠️ 無法取得設定檔案位置，YAML 未儲存！");
        }
    }

    /// **載入 YAML**
    fn load() -> Self {
        if let Some(path) = Self::get_config_path() {
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
        } else {
            eprintln!("⚠️ 無法取得設定檔案位置，使用預設值！");
            Self { data: 3.0 }
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
            config: DataConfig::load(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("輸入數值:");
            ui.add(egui::DragValue::new(&mut self.config.data));

            if ui.button("儲存到 YAML").clicked() {
                self.config.save();
            }

            if ui.button("從 YAML 載入").clicked() {
                self.config = DataConfig::load();
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
