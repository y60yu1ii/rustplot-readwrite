use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;

pub fn load_custom_font(ctx: &Context) {
    let font_path = "assets/NotoSansTC-Medium.ttf"; // 確保字型放在這個路徑

    let mut fonts = FontDefinitions {
        font_data: BTreeMap::new(),
        families: BTreeMap::new(),
    };

    if let Ok(font_data) = fs::read(font_path) {
        fonts.font_data.insert(
            "custom_font".to_string(),
            Arc::new(FontData::from_owned(font_data)),
        );

        fonts
            .families
            .insert(FontFamily::Proportional, vec!["custom_font".to_string()]);
        fonts
            .families
            .insert(FontFamily::Monospace, vec!["custom_font".to_string()]);

        ctx.set_fonts(fonts);
    } else {
        eprintln!("⚠️ 載入字型失敗: 找不到 `{}`", font_path);
    }
}
