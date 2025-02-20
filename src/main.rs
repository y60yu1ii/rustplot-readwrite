mod font_loader; // 引入字型模組

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct DataConfig {
    data: f32,
}

impl DataConfig {
    fn save(&self) {
        let yaml = serde_yaml::to_string(self).unwrap();
        fs::write("data.yaml", yaml).expect("無法寫入檔案");
    }

    fn load() -> Self {
        match fs::read_to_string("data.yaml") {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or(Self { data: 3.0 }),
            Err(_) => Self { data: 3.0 }, // 預設值
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
