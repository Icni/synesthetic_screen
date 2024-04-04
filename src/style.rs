pub fn load_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    use egui::{Color32, Rounding};
    style.visuals.panel_fill = Color32::BLACK;
    style.visuals.window_rounding = Rounding::same(5.0);
    style.visuals.window_fill = Color32::BLACK;

    use egui::{TextStyle::*, FontId, FontFamily};
    style.text_styles = [
        (Heading, FontId::new(18., FontFamily::Proportional)),
        (Body, FontId::new(14., FontFamily::Proportional)),
        (Button, FontId::new(14., FontFamily::Proportional)),
        (Monospace, FontId::new(14., FontFamily::Monospace)),
        (Small, FontId::new(10., FontFamily::Proportional)),
    ].into();

    ctx.set_style(style);

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Unageo".to_string(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Unageo/fonts/ttf/Unageo-Regular.ttf"))
    );
    fonts.font_data.insert(
        "TJF Optik".to_string(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/TJF Optik/TjfOptikBlack.ttf"))
    );

    fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Unageo".to_string());
    fonts.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "TJF Optik".to_string());

    ctx.set_fonts(fonts);
}
