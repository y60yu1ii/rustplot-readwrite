#![windows_subsystem = "windows"]
mod font_loader;
mod save_load;

use eframe::{egui, NativeOptions};
use egui::{IconData, ViewportBuilder};
use image::ImageReader;
use rand::Rng;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct UIConfig {
    components: Vec<UIComponent>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")] // ✅ 確保 YAML `type` 正確
enum UIComponent {
    Label {
        key: String,
        text: Option<String>,
        unit: Option<String>,
    },
    Button {
        key: String,
        text: String,
    },
    Input {
        label: String,
    },
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            components: vec![
                UIComponent::Label {
                    key: "default_lb".to_string(),
                    text: Some("預設標籤".to_string()),
                    unit: Some("unit".to_string()),
                },
                UIComponent::Button {
                    key: "default_btn".to_string(),
                    text: "預設按鈕".to_string(),
                },
                UIComponent::Input {
                    label: "預設輸入".to_string(),
                },
            ],
        }
    }
}

struct MyApp {
    config: UIConfig,
    config_path: Option<PathBuf>,
    label_data: Arc<Mutex<HashMap<String, f64>>>, // ✅ 儲存 Label 數據
}

impl MyApp {
    fn new(ctx: &egui::Context) -> Self {
        font_loader::load_custom_font(ctx);

        let config_path = save_load::DataConfig::get_config_path();
        let config = if let Some(ref path) = config_path {
            Self::load_config(path)
        } else {
            UIConfig::default()
        };

        let label_data = Arc::new(Mutex::new(HashMap::new()));

        let mut app = Self {
            config,
            config_path,
            label_data: label_data.clone(),
        };

        app.initialize_label_data(); // ✅ 初始化 Label 數據
        app.start_data_update_loop(label_data, ctx.clone()); // ✅ 啟動數據更新
        app
    }

    fn initialize_label_data(&mut self) {
        let mut label_data = self.label_data.lock().unwrap();
        for component in &self.config.components {
            if let UIComponent::Label { key, .. } = component {
                label_data.entry(key.clone()).or_insert(0.0); // ✅ 設定預設值
            }
        }
    }

    fn start_data_update_loop(
        &mut self,
        label_data: Arc<Mutex<HashMap<String, f64>>>,
        ctx: egui::Context,
    ) {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            let mut rng = rand::rng();
            let mut data_lock = label_data.lock().unwrap();

            for key in data_lock.keys().cloned().collect::<Vec<String>>() {
                let value = rng.random_range(50.0..500.0);

                data_lock.insert(key, value);
            }

            ctx.request_repaint();
        });
    }

    fn load_config(path: &PathBuf) -> UIConfig {
        match fs::read_to_string(path) {
            Ok(content) => match serde_yaml::from_str(&content) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("⚠️ YAML 解析錯誤: {:?}", err);
                    UIConfig::default()
                }
            },
            Err(_) => UIConfig::default(),
        }
    }

    fn save_config(&self, path: Option<PathBuf>) {
        let path = path.unwrap_or_else(|| {
            self.config_path
                .clone()
                .unwrap_or(PathBuf::from("data.yaml"))
        });

        if let Ok(yaml) = serde_yaml::to_string(&self.config) {
            if let Err(e) = fs::write(&path, yaml) {
                eprintln!("⚠️ 無法寫入設定檔: {:?}", e);
            } else {
                println!("✅ 設定已儲存至 {:?}", path);
            }
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.menu_button("檔案", |ui| {
            if ui.button("儲存到 YAML").clicked() {
                self.save_config(None);
            }

            if ui.button("從 YAML 載入").clicked() {
                if let Some(ref path) = self.config_path {
                    self.config = Self::load_config(path);
                    self.initialize_label_data(); // ✅ 重新初始化數據
                    ctx.request_repaint();
                }
            }

            ui.separator();

            if ui.button("從外部檔案選擇 YAML").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    println!("📂 選擇 YAML 檔案: {:?}", path);
                    self.config = Self::load_config(&path);
                    self.config_path = save_load::DataConfig::get_config_path();
                    self.save_config(None);
                    self.initialize_label_data();
                    ctx.request_repaint();
                }
            }

            if ui.button("選擇儲存 YAML 位置").clicked() {
                if let Some(path) = FileDialog::new()
                    .set_file_name("ui_config.yaml")
                    .save_file()
                {
                    println!("💾 儲存 YAML 到: {:?}", path);
                    self.save_config(Some(path));
                }
            }
        });
    }

    fn show_ui_from_config(&mut self, ui: &mut egui::Ui) {
        for component in &self.config.components {
            match component {
                UIComponent::Label { key, text, unit } => {
                    let value = {
                        let data = self.label_data.lock().unwrap();
                        *data.get(key).unwrap_or(&0.0)
                    };
                    let display_text = format!(
                        "{}: {:.2} {}",
                        text.as_deref().unwrap_or("Label"),
                        value,
                        unit.as_deref().unwrap_or("")
                    );
                    ui.label(display_text);
                }
                UIComponent::Button { key, text } => {
                    if ui.button(text).clicked() {
                        println!("🔘 按鈕 `{}` 被點擊！Key: {}", text, key);
                    }
                }
                UIComponent::Input { label } => {
                    let mut input_text = String::new();
                    ui.horizontal(|ui| {
                        ui.label(label);
                        ui.text_edit_singleline(&mut input_text);
                    });
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_menu_bar(ui, ctx);
            ui.separator();
            self.show_ui_from_config(ui);
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_icon().ok();

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_icon(icon.unwrap_or_else(|| IconData {
                rgba: vec![0; 256 * 256 * 4], // 預設透明圖標
                width: 256,
                height: 256,
            })),
        ..Default::default()
    };

    eframe::run_native(
        "動態 YAML UI",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(&cc.egui_ctx)))), // ✅ 確保回傳 `Result`
    )?;

    Ok(())
}

fn load_icon() -> Result<IconData, Box<dyn std::error::Error>> {
    let image_bytes = include_bytes!("../assets/icon.png");
    let image = ImageReader::new(Cursor::new(image_bytes))
        .with_guessed_format()?
        .decode()?
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Ok(IconData {
        rgba,
        width,
        height,
    })
}
