use bevy::prelude::*;
use bevy_egui::{EguiPlugin, egui};
use std::collections::HashMap;
use std::sync::Mutex;

const NOTO_SANS_SC: &[u8] = include_bytes!("../../assets/fonts/NotoSansSC-Regular.otf");

/// Noto Sans SC 字体标识符
pub const FONT_NOTO_SANS_SC: &str = "noto_sans_sc";

/// Bevy 插件：在 Startup 阶段向 EGUI 注册字体
pub struct FontCenterPlugin;

impl Plugin for FontCenterPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.init_resource::<FontRegistry>()
            .add_systems(Startup, setup_egui_fonts);
    }
}

/// 已注册字体的运行时查询表
#[derive(Resource, Default)]
pub struct FontRegistry {
    fonts: HashMap<String, FontMeta>,
}

/// 单条字体的元数据
#[derive(Clone, Debug)]
pub struct FontMeta {
    /// 显示名称
    pub name: String,
    /// 字体来源
    pub source: FontSource,
}

/// 字体数据来源
#[derive(Clone, Debug)]
pub enum FontSource {
    /// 编译期嵌入
    Embedded,
    /// 运行时文件路径
    FileSystem(String),
}

impl FontRegistry {
    /// 检查指定名称的字体是否已注册
    pub fn has(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }

    /// 获取字体元数据
    pub fn get(&self, name: &str) -> Option<&FontMeta> {
        self.fonts.get(name)
    }

    /// 返回所有已注册字体的名称列表
    pub fn list(&self) -> Vec<&str> {
        self.fonts.keys().map(|s| s.as_str()).collect()
    }
}

static FONT_INIT_GUARD: Mutex<bool> = Mutex::new(false);

fn setup_egui_fonts(mut contexts: bevy_egui::EguiContexts, mut registry: ResMut<FontRegistry>) {
    let ctx = contexts.ctx_mut();

    {
        let mut guard = FONT_INIT_GUARD.lock().unwrap();
        if *guard {
            return;
        }
        *guard = true;
    }

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        FONT_NOTO_SANS_SC.to_owned(),
        egui::FontData::from_owned(NOTO_SANS_SC.to_vec()),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, FONT_NOTO_SANS_SC.to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(FONT_NOTO_SANS_SC.to_owned());

    registry.fonts.insert(
        FONT_NOTO_SANS_SC.to_owned(),
        FontMeta {
            name: "Noto Sans SC".to_owned(),
            source: FontSource::Embedded,
        },
    );

    ctx.set_fonts(fonts);
    apply_dark_theme(ctx);
}

/// 将 EGUI 全局视觉主题设为暗色
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = egui::Rounding::same(8.0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 16.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(128),
    };
    visuals.window_fill = egui::Color32::from_rgb(28, 28, 32);
    visuals.panel_fill = egui::Color32::from_rgb(24, 24, 28);
    visuals.extreme_bg_color = egui::Color32::from_rgb(16, 16, 20);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(48, 48, 56);
    visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(200, 200, 210);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(64, 64, 80);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 100);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(56, 56, 72);
    visuals.selection.bg_fill = egui::Color32::from_rgb(70, 100, 140);
    visuals.hyperlink_color = egui::Color32::from_rgb(100, 160, 220);
    visuals.faint_bg_color = egui::Color32::from_rgb(36, 36, 44);
    visuals.code_bg_color = egui::Color32::from_rgb(40, 40, 50);
    ctx.set_visuals(visuals);
}
