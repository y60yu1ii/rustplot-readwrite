#![windows_subsystem = "windows"]
mod font_loader;
mod save_load;

use eframe::{egui, NativeOptions};
use egui::{IconData, ViewportBuilder};
use image::ImageReader;
use regex::Regex;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct UIConfig {
    components: Vec<UIComponent>,
    canbus_config: Vec<CanBusConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CanBusConfig {
    key: String,
    id: u32,
    index: u8,
    len: u8,
    endian: u8,
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
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
    Graph {
        key: String,
    },
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            components: vec![UIComponent::Label {
                key: "lb0".to_string(),
                text: Some("預設標籤".to_string()),
                unit: Some("unit".to_string()),
            }],
            canbus_config: vec![CanBusConfig {
                key: "lb0".to_string(),
                id: 0x00,
                index: 0,
                len: 2,
                endian: 0,
                r#type: "float32".to_string(),
            }],
        }
    }
}

struct MyApp {
    config: UIConfig,
    config_path: Option<PathBuf>,
    label_data: Arc<Mutex<HashMap<String, f64>>>,
    graph_data: Arc<Mutex<HashMap<String, VecDeque<[f64; 2]>>>>,
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

        let data_store = Arc::new(Mutex::new(HashMap::new())); // ✅ This will be our main data storage
        let graph_data = Arc::new(Mutex::new(HashMap::new()));

        let mut app = Self {
            config,
            config_path,
            label_data: data_store.clone(),
            graph_data: graph_data.clone(),
        };

        app.initialize_data_store();
        app.initialize_label_data();
        app.start_data_update_loop(
            app.config.canbus_config.clone(),
            data_store,
            graph_data,
            ctx.clone(),
        ); // ✅ Start updating

        app
    }

    fn initialize_label_data(&mut self) {
        let mut label_data = self.label_data.lock().unwrap();
        let mut graph_data = self.graph_data.lock().unwrap();

        for component in &self.config.components {
            match component {
                UIComponent::Label { key, .. } | UIComponent::Graph { key } => {
                    label_data.entry(key.clone()).or_insert(0.0);
                    graph_data
                        .entry(key.clone())
                        .or_insert_with(|| VecDeque::with_capacity(200));
                }
                _ => {}
            }
        }

        for canbus in &self.config.canbus_config {
            label_data.entry(canbus.key.clone()).or_insert(0.0);
        }
    }

    fn initialize_data_store(&mut self) {
        let mut data_store = self.label_data.lock().unwrap();
        let mut graph_store = self.graph_data.lock().unwrap();

        for canbus in &self.config.canbus_config {
            data_store.entry(canbus.key.clone()).or_insert(0.0);
            graph_store
                .entry(canbus.key.clone())
                .or_insert_with(|| VecDeque::with_capacity(200));
        }
    }

    fn start_data_update_loop(
        &mut self,
        canbus_config: Vec<CanBusConfig>,
        data_store: Arc<Mutex<HashMap<String, f64>>>,
        graph_data: Arc<Mutex<HashMap<String, VecDeque<[f64; 2]>>>>,
        ctx: egui::Context,
    ) {
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            let elapsed_time = ctx.input(|i| i.time);

            {
                let mut data_lock = data_store.lock().unwrap();
                let mut graph_lock = graph_data.lock().unwrap();

                for config in &canbus_config {
                    let key = config.key.clone();

                    let idx = extract_number(&key).unwrap_or(0) as f64;

                    let value = data_lock.entry(key.clone()).or_insert(idx);
                    let sign = if rand::random() { 1.0 } else { -1.0 };
                    *value += idx * sign;

                    let entry = graph_lock
                        .entry(key.clone())
                        .or_insert_with(|| VecDeque::with_capacity(200));

                    entry.push_back([elapsed_time, *value]);

                    if entry.len() > 200 {
                        entry.pop_front();
                    }
                }
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
        let mut graph_count = HashMap::new();
        for component in &self.config.components {
            match component {
                UIComponent::Graph { key } => {
                    use egui_plot::{Line, Plot, PlotPoints};

                    let count = graph_count.entry(key.clone()).or_insert(0);
                    *count += 1; // Increment the count for this key

                    let unique_id = format!("graph_{}_{}", key, count); // Unique ID per instance

                    let data_points = {
                        let graph_data = self.graph_data.lock().unwrap();
                        graph_data.get(key).cloned().unwrap_or_default()
                    };

                    if !data_points.is_empty() {
                        Plot::new(unique_id).height(120.0).show(ui, |plot_ui| {
                            let line =
                                Line::new(PlotPoints::from_iter(data_points.iter().copied()));
                            plot_ui.line(line);
                        });
                    }
                }
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

fn extract_number(key: &str) -> Option<u32> {
    let re = Regex::new(r"lb(\d+)").unwrap();
    if let Some(captures) = re.captures(key) {
        captures.get(1)?.as_str().parse::<u32>().ok()
    } else {
        None
    }
}
