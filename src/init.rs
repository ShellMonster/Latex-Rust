//! 初始化模块：负责加载字体并提供全局访问入口

use fontdue::{Font, FontSettings}; // 引入 fontdue 中的字体类型与配置
use once_cell::sync::Lazy; // 引入 Lazy，确保字体只会加载一次
use usvg::fontdb::Database; // 引入字体数据库，供 usvg/resvg 使用

use crate::error::RenderError; // 引入项目内自定义的错误类型

/// 在编译期把字体文件打包进二进制，避免运行时找不到资源
static CMATH_BYTES: &[u8] = include_bytes!("../fonts/latinmodern-math.otf"); // Computer Modern 系列的数学字体

struct FontAssets {
    fontdue: Font,
    database: Database,
}

/// 用于保存懒加载后的字体对象，失败时记录错误
static FONT_ASSETS: Lazy<Result<FontAssets, RenderError>> = Lazy::new(|| {
    let font = Font::from_bytes(CMATH_BYTES, FontSettings::default())
        .map_err(|err| RenderError::FontLoadError(format!("无法解析字体: {err}")))?;

    let mut db = Database::new();
    db.load_font_data(CMATH_BYTES.to_vec());
    if db.is_empty() {
        return Err(RenderError::FontLoadError(
            "字体数据库未能加载任何字体面".into(),
        ));
    }
    db.set_sans_serif_family(primary_font_family().to_string());
    db.set_serif_family(primary_font_family().to_string());
    db.set_monospace_family(primary_font_family().to_string());

    Ok(FontAssets {
        fontdue: font,
        database: db,
    })
});

/// 保证字体只加载一次，并且在使用前就绪
pub fn ensure_fonts_loaded() -> Result<(), RenderError> {
    match &*FONT_ASSETS {
        Ok(_) => Ok(()),
        Err(err) => Err(err.clone()),
    }
}

/// 提供默认字体的引用，供布局和渲染模块使用
pub fn default_font() -> Result<&'static Font, RenderError> {
    match &*FONT_ASSETS {
        Ok(assets) => Ok(&assets.fontdue),
        Err(err) => Err(err.clone()),
    }
}

/// 提供给 usvg/resvg 使用的字体数据库
pub fn font_database() -> Result<&'static Database, RenderError> {
    match &*FONT_ASSETS {
        Ok(assets) => Ok(&assets.database),
        Err(err) => Err(err.clone()),
    }
}

/// 返回默认使用的字体名，便于 SVG 设置字体族
pub fn default_font_family() -> &'static str {
    "'Latin Modern Math', 'Latin Modern Roman', 'Computer Modern', serif" // Computer Modern 家族，符合需求约束
}

/// 返回主字体族名称，供解析 pipeline 设置默认字体
pub fn primary_font_family() -> &'static str {
    "Latin Modern Math"
}

/// 返回渲染时使用的默认字号，单位为像素
pub fn default_font_size() -> f32 {
    28.0 // 初始字号选择 28px，后续可以在布局模块里按需调整
}
