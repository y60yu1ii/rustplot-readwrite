use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use include_dir::{include_dir, Dir};
use std::sync::Arc;

static ASSETS: Dir = include_dir!("assets/");

pub fn load_custom_font(ctx: &Context) {
    // 內嵌字體檔案
    let font_data = ASSETS
        .get_file("NotoSansTC-Medium.ttf")
        .expect("⚠️ 無法載入內嵌字體 `NotoSansTC-Medium.ttf`！請確認 `assets/` 內的檔案名稱正確")
        .contents();

    let mut fonts = FontDefinitions::default();

    // ✅ 設定 `custom_font` 為內嵌的 `NotoSansTC-Medium.ttf`
    fonts.font_data.insert(
        "custom_font".to_string(),
        Arc::new(FontData::from_owned(font_data.to_vec())),
    );

    // 設定 `Proportional` & `Monospace` 使用 `custom_font`
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("custom_font".to_string());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("custom_font".to_string());

    ctx.set_fonts(fonts);
}
