//! 渲染模块：把排版结果转成最终的 SVG 字符串

use crate::config::{should_embed_font, svg_text_mode, SvgTextMode}; // 引入输出模式配置
use crate::error::RenderError; // 引入错误类型
use crate::init;
use crate::layout::LayoutPlan; // 引入排版阶段的输出数据

use resvg::Tree as ResvgTree;
use std::borrow::Cow;
use std::fmt::Write;
use usvg::{Options as UsvgOptions, TreeParsing, TreeTextToPath, TreeWriting, XmlOptions};

/// 把布局信息转换为 SVG 字符串
pub fn render_svg_document(plan: &LayoutPlan) -> Result<String, RenderError> {
    let base_svg = build_base_svg(plan);

    if matches!(svg_text_mode(), SvgTextMode::Text) {
        // 默认返回文本版 SVG，避免体积膨胀
        return Ok(base_svg);
    }

    let mut opts = UsvgOptions::default();
    opts.font_family = init::primary_font_family().to_string();
    opts.font_size = init::default_font_size();

    let mut tree = usvg::Tree::from_str(&base_svg, &opts)
        .map_err(|err| RenderError::RenderFailure(format!("usvg 解析失败: {err}")))?;

    let font_db = init::font_database()?;
    tree.convert_text(font_db);

    let render_tree = ResvgTree::from_usvg(&tree);
    tree.size = render_tree.size;
    tree.view_box = render_tree.view_box;

    let svg = tree.to_string(&XmlOptions::default());
    Ok(svg)
}

fn build_base_svg(plan: &LayoutPlan) -> String {
    let safe_width = plan.width.max(1.0);
    let safe_height = plan.height.max(1.0);
    let estimated = (plan.items.len() + plan.lines.len() + plan.paths.len()) * 96 + 256;
    let mut svg = String::with_capacity(estimated);
    let _ = write!(
        &mut svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.2}" height="{height:.2}" viewBox="0 0 {width:.2} {height:.2}" preserveAspectRatio="xMinYMin meet">"#,
        width = safe_width,
        height = safe_height
    );

    if should_embed_font() {
        embed_font_face(&mut svg, plan.font_family);
    }

    if !plan.lines.is_empty() {
        svg.push_str("<g stroke=\"#000000\" fill=\"none\">");
        for line in &plan.lines {
            let _ = write!(
                &mut svg,
                r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke-width="{:.2}" stroke-linecap="round"/>"#,
                line.x1, line.y1, line.x2, line.y2, line.stroke_width
            );
        }
        svg.push_str("</g>");
    }

    if !plan.paths.is_empty() {
        svg.push_str("<g>");
        for path in &plan.paths {
            let fill = path.fill.unwrap_or("none");
            let stroke = path.stroke.unwrap_or("#000000");
            let _ = write!(
                &mut svg,
                r#"<path d="{}" fill="{}" stroke="{}""#,
                path.d, fill, stroke
            );
            if let Some(width) = path.stroke_width {
                let _ = write!(&mut svg, r#" stroke-width="{:.2}""#, width);
            }
            if let Some(cap) = path.stroke_linecap {
                let _ = write!(&mut svg, r#" stroke-linecap="{}""#, cap);
            }
            if let Some(join) = path.stroke_linejoin {
                let _ = write!(&mut svg, r#" stroke-linejoin="{}""#, join);
            }
            if path.x != 0.0 || path.y != 0.0 {
                let _ = write!(
                    &mut svg,
                    r#" transform="translate({:.2} {:.2})""#,
                    path.x, path.y
                );
            }
            svg.push_str("/>");
        }
        svg.push_str("</g>");
    }

    if !plan.items.is_empty() {
        svg.push_str("<g fill=\"#000000\">");
        for item in &plan.items {
            let escaped = escape_text(&item.text);
            let _ = write!(
                &mut svg,
                r#"<text x="{:.2}" y="{:.2}" font-family="{}" font-size="{:.2}">{}"#,
                item.x, item.y, plan.font_family, item.font_size, escaped
            );
            svg.push_str("</text>");
        }
        svg.push_str("</g>");
    }

    svg.push_str("</svg>");
    svg
}

fn embed_font_face(svg: &mut String, font_family: &str) {
    if !font_family.contains("Latin Modern Math") {
        return;
    }
    if svg.contains("@font-face") {
        return;
    }
    svg.push_str("<defs><style>@font-face { font-family: 'Latin Modern Math'; src: url(\"data:font/woff2;base64,");
    svg.push_str(FONT_EMBED);
    svg.push_str("\") format('woff2'); font-weight: normal; font-style: normal; }</style></defs>");
}

const FONT_EMBED: &str = include_str!("../fonts/latinmodern-math.woff2.b64");

/// 替换文本中的 XML 关键字符，避免产生非法 SVG
fn escape_text(input: &str) -> Cow<'_, str> {
    if !input
        .bytes()
        .any(|b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\''))
    {
        return Cow::Borrowed(input);
    }

    let mut escaped = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    Cow::Owned(escaped)
}
