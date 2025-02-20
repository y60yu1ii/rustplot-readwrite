#![windows_subsystem = "windows"]
mod font_loader;
mod graph;
mod save_load; // ✅ 引入 `graph.rs`

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use graph::{Graph, GraphType}; // ✅ 使用 Graph 模組
use rfd::FileDialog;
use save_load::DataConfig;
use std::time::{Duration, Instant};

const NUM_TRIANGLE_GRAPHS: usize = 4; // ✅ 4 個遞增遞減波
const NUM_SIN_GRAPHS: usize = 6; // ✅ 6 個 Sin 波

struct MyApp {
    config: DataConfig,
    start_time: Instant,
    graphs: Vec<Graph>,
}

impl MyApp {
    fn new(ctx: &egui::Context) -> Self {
        font_loader::load_custom_font(ctx);
        let mut graphs = Vec::new();

        // ✅ 4 個遞增遞減波
        for _ in 0..NUM_TRIANGLE_GRAPHS {
            graphs.push(Graph::new(GraphType::Triangle));
        }

        // ✅ 6 個 Sin 波
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

        // 更新所有圖表數據
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

            // ✅ 用 ScrollArea 來讓圖表可以滾動
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, graph) in self.graphs.iter().enumerate() {
                    let label = match graph.graph_type {
                        GraphType::Triangle => format!("圖表 {} (三角波)", index + 1),
                        GraphType::SinWave => format!("圖表 {} (Sin 波)", index + 1),
                    };
                    ui.label(label);

                    Plot::new(format!("real_time_plot_{}", index))
                        .height(120.0) // ✅ 調高單個圖表高度，避免擠在一起
                        .allow_scroll(false) // ❌ 禁止滑鼠滾動影響 Plot
                        .allow_drag(false) // ❌ 禁止拖動
                        .allow_zoom(false) // ❌ 禁止縮放
                        .show(ui, |plot_ui| {
                            let line = Line::new(PlotPoints::from_iter(
                                graph.data.iter().map(|&(x, y)| [x, y]), // ✅ 轉換成 `[f64; 2]`
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
    let mut options = eframe::NativeOptions::default();
    options.viewport.inner_size = Some([800.0, 900.0].into()); // ✅ 改小視窗大小，測試 ScrollBar

    eframe::run_native(
        "時間數據圖表",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(&cc.egui_ctx)))),
    )?;

    Ok(())
}
