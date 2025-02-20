#![windows_subsystem = "windows"]
mod font_loader;
mod graph;
mod save_load;

use eframe::{egui, NativeOptions};
use egui::{IconData, ViewportBuilder};
use egui_plot::{Line, Plot, PlotPoints};
use graph::{Graph, GraphType};
use image::ImageReader;
use rfd::FileDialog;
use save_load::DataConfig;
use std::fs;
use std::io::Cursor;
use std::time::{Duration, Instant};

const NUM_TRIANGLE_GRAPHS: usize = 4;
const NUM_SIN_GRAPHS: usize = 6;

struct MyApp {
    config: DataConfig,
    start_time: Instant,
    graphs: Vec<Graph>,
}

impl MyApp {
    fn new(ctx: &egui::Context) -> Self {
        font_loader::load_custom_font(ctx);
        let mut graphs = Vec::new();

        for _ in 0..NUM_TRIANGLE_GRAPHS {
            graphs.push(Graph::new(GraphType::Triangle));
        }

        for _ in 0..NUM_SIN_GRAPHS {
            graphs.push(Graph::new(GraphType::SinWave));
        }

        Self {
            config: DataConfig::load(None),
            start_time: Instant::now(),
            graphs,
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("檔案", |ui| {
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
                if let Some(path) = FileDialog::new().set_file_name("data.yaml").save_file() {
                    println!("💾 儲存 YAML 到: {:?}", path);
                    self.config.save(Some(path));
                }
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let elapsed = self.start_time.elapsed().as_secs_f64();

        for graph in &mut self.graphs {
            graph.update(elapsed);
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.show_menu_bar(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.separator();
            ui.heading("即時數據圖表");

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, graph) in self.graphs.iter().enumerate() {
                    let label = match graph.graph_type {
                        GraphType::Triangle => format!("圖表 {} (三角波)", index + 1),
                        GraphType::SinWave => format!("圖表 {} (Sin 波)", index + 1),
                    };
                    ui.label(label);

                    Plot::new(format!("real_time_plot_{}", index))
                        .height(120.0)
                        .allow_scroll(false)
                        .allow_drag(false)
                        .allow_zoom(false)
                        .show(ui, |plot_ui| {
                            let line = Line::new(PlotPoints::from_iter(
                                graph.data.iter().map(|&(x, y)| [x, y]),
                            ));
                            plot_ui.line(line);
                        });
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_icon("assets/icon.png")?;

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 900.0])
            .with_icon(icon), // ✅ 設定應用程式 Icon
        ..Default::default()
    };

    eframe::run_native(
        "時間數據圖表",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(&cc.egui_ctx)))),
    )?;

    Ok(())
}

fn load_icon(path: &str) -> Result<IconData, Box<dyn std::error::Error>> {
    let image_bytes = fs::read(path)?;
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
