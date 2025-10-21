//! 渲染配置模块：管理 SVG 输出模式等可调参数

use once_cell::sync::Lazy; // 延迟读取环境变量
use std::env; // 读取环境变量
use std::sync::atomic::{AtomicBool, Ordering as BoolOrdering};
use std::sync::atomic::{AtomicU8, Ordering}; // 存储全局覆盖开关

/// SVG 输出模式：保留 `<text>` 还是转换为矢量路径
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SvgTextMode {
    /// 默认模式，保留 `<text>` 节点以缩小文件体积
    Text,
    /// 把文字转换成 `<path>`，确保无字体依赖
    Paths,
}

// 0: 未覆盖，1: Text，2: Paths
static MODE_OVERRIDE: AtomicU8 = AtomicU8::new(0);

/// 环境变量 `FORMULA_SVG_MODE` 的默认设置
static ENV_DEFAULT: Lazy<SvgTextMode> = Lazy::new(|| match env::var("FORMULA_SVG_MODE") {
    Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
        "path" | "paths" => SvgTextMode::Paths,
        "text" => SvgTextMode::Text,
        _ => SvgTextMode::Text,
    },
    Err(_) => SvgTextMode::Text,
});

/// 获取当前 SVG 输出模式（覆盖优先于环境变量）
pub fn svg_text_mode() -> SvgTextMode {
    match MODE_OVERRIDE.load(Ordering::Relaxed) {
        1 => SvgTextMode::Text,
        2 => SvgTextMode::Paths,
        _ => *ENV_DEFAULT,
    }
}

/// 允许在运行时覆盖 SVG 输出模式；`None` 表示还原为默认设置
pub fn override_svg_text_mode(mode: Option<SvgTextMode>) {
    let value = match mode {
        Some(SvgTextMode::Text) => 1,
        Some(SvgTextMode::Paths) => 2,
        None => 0,
    };
    MODE_OVERRIDE.store(value, Ordering::Relaxed);
}

static EMBED_FONT_OVERRIDE: AtomicBool = AtomicBool::new(false);

static EMBED_FONT_DEFAULT: Lazy<bool> = Lazy::new(|| match env::var("FORMULA_SVG_EMBED_FONT") {
    Ok(value) => matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    ),
    Err(_) => false,
});

pub fn should_embed_font() -> bool {
    if EMBED_FONT_OVERRIDE.load(BoolOrdering::Relaxed) {
        return true;
    }
    *EMBED_FONT_DEFAULT
}

#[allow(dead_code)]
pub fn override_embed_font(enable: bool) {
    EMBED_FONT_OVERRIDE.store(enable, BoolOrdering::Relaxed);
}
