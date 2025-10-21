//! 排版模块：将语法树转换为可直接绘制的布局数据

use crate::ast::{
    AstNode, DecorationKind, Delimiter, LargeOperatorNode, ParsedFormula, SpecialSymbol,
};
use crate::error::RenderError; // 引入统一错误类型
use crate::init; // 字体初始化模块 // 引入语法树结构

use fontdue::{Font, Metrics as GlyphMetrics}; // 用于访问字体度量及字形指标
use std::cell::RefCell;
use std::collections::HashMap;
use std::thread_local;

/// SVG 绘制所需的文字片段
#[derive(Debug, Clone)]
pub struct RenderItem {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
}

/// SVG 中需要绘制的直线（用于分数横线、根号顶线等）
#[derive(Debug, Clone)]
pub struct RenderLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub stroke_width: f32,
}

/// SVG 中需要绘制的路径（自定义括号、装饰等）
#[derive(Debug, Clone)]
pub struct RenderPath {
    pub d: String,
    pub x: f32,
    pub y: f32,
    pub fill: Option<&'static str>,
    pub stroke: Option<&'static str>,
    pub stroke_width: Option<f32>,
    pub stroke_linecap: Option<&'static str>,
    pub stroke_linejoin: Option<&'static str>,
}

/// 排版结果：包含整体尺寸以及所有绘制元素
#[derive(Debug, Clone)]
pub struct LayoutPlan {
    pub width: f32,
    pub height: f32,
    pub font_family: &'static str,
    pub items: Vec<RenderItem>,
    pub lines: Vec<RenderLine>,
    pub paths: Vec<RenderPath>,
}

/// 对外入口：将解析结果转换为布局信息
pub fn layout(parsed: &ParsedFormula) -> Result<LayoutPlan, RenderError> {
    let font = init::default_font()?; // 先确保字体加载成功
    let font_family = init::default_font_family();
    let base_font_size = init::default_font_size();

    let root_box = layout_node(&parsed.ast, base_font_size, &font)?; // 递归生成布局盒
    let padding = base_font_size * 0.2; // 留出一定的边距，避免字符被裁剪

    let mut items = root_box.items;
    offset_items(&mut items, padding, padding); // 整体平移，保证四周留白

    let mut lines = root_box.lines;
    offset_lines(&mut lines, padding, padding);

    let mut paths = root_box.paths;
    offset_paths(&mut paths, padding, padding);

    Ok(LayoutPlan {
        width: root_box.width + padding * 2.0,
        height: root_box.height + padding * 2.0,
        font_family,
        items,
        lines,
        paths,
    })
}

/// 布局盒：中间计算过程中使用，包含尺寸和元素集合
#[derive(Debug, Clone)]
struct LayoutBox {
    width: f32,
    height: f32,
    baseline: f32,
    script_policy: ScriptPolicy,
    italic_correction: f32,
    items: Vec<RenderItem>,
    lines: Vec<RenderLine>,
    paths: Vec<RenderPath>,
}

#[derive(Debug, Clone, Copy)]
enum ScriptPolicy {
    /// 默认脚本放在右上角/右下角
    Right,
    /// 用于求和、积分等大型算符，脚本需要居中放在上方/下方
    AboveBelow,
}

fn layout_node(node: &AstNode, font_size: f32, font: &Font) -> Result<LayoutBox, RenderError> {
    match node {
        AstNode::Text(content) => layout_text(content, font_size, font),
        AstNode::Group(children) => layout_group(children, font_size, font),
        AstNode::Fraction {
            numerator,
            denominator,
        } => layout_fraction(numerator, denominator, font_size, font),
        AstNode::Sqrt { value } => layout_sqrt(value, font_size, font),
        AstNode::Delimited { left, inner, right } => {
            layout_delimited(left, inner, right, font_size, font)
        }
        AstNode::LargeOperator(node) => layout_large_operator(node, font_size, font),
        AstNode::Matrix(rows) => layout_matrix(rows, font_size, font),
        AstNode::Decorated { base, decoration } => {
            layout_decorated(base, *decoration, font_size, font)
        }
        AstNode::Scripts {
            base,
            superscript,
            subscript,
        } => layout_scripts(
            base,
            superscript.as_deref(),
            subscript.as_deref(),
            font_size,
            font,
        ),
        AstNode::Symbol(symbol) => layout_symbol(*symbol, font_size, font),
    }
}

fn layout_text(content: &str, font_size: f32, font: &Font) -> Result<LayoutBox, RenderError> {
    let (ascent, descent, _) = line_metrics(font, font_size);
    let mut width = 0.0f32;
    let mut italic_correction = 0.0f32;
    for ch in content.chars() {
        let metrics = cached_metrics(font, ch, font_size);
        width += metrics.advance_width;
        italic_correction = glyph_italic_correction(&metrics);
    }
    let baseline = ascent;
    let height = ascent + descent;
    let item = RenderItem {
        text: content.to_string(),
        x: 0.0,
        y: baseline,
        font_size,
    };
    Ok(LayoutBox {
        width,
        height,
        baseline,
        script_policy: ScriptPolicy::Right,
        italic_correction,
        items: vec![item],
        lines: Vec::new(),
        paths: Vec::new(),
    })
}

fn layout_symbol(
    symbol: SpecialSymbol,
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    let (ch, scale, policy) = match symbol {
        SpecialSymbol::Sum => ('∑', 1.35, ScriptPolicy::AboveBelow),
        SpecialSymbol::Product => ('∏', 1.35, ScriptPolicy::AboveBelow),
        SpecialSymbol::Integral => ('∫', 1.45, ScriptPolicy::AboveBelow),
    };

    let display_size = font_size * scale;
    let (ascent, descent, _) = line_metrics(font, display_size);
    let metrics = cached_metrics(font, ch, display_size);
    let width = metrics.advance_width.max(display_size * 0.6);
    let baseline = ascent;
    let height = ascent + descent;

    let item = RenderItem {
        text: ch.to_string(),
        x: 0.0,
        y: baseline,
        font_size: display_size,
    };

    Ok(LayoutBox {
        width,
        height,
        baseline,
        script_policy: policy,
        italic_correction: 0.0,
        items: vec![item],
        lines: Vec::new(),
        paths: Vec::new(),
    })
}

fn layout_large_operator(
    node: &LargeOperatorNode,
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    let effective_size = font_size * node.scale;
    let (ascent, descent, _) = line_metrics(font, effective_size);
    let width = measure_text_width(node.content.as_str(), effective_size, font);
    Ok(LayoutBox {
        width,
        height: ascent + descent,
        baseline: ascent,
        script_policy: ScriptPolicy::AboveBelow,
        italic_correction: 0.0,
        items: vec![RenderItem {
            text: node.content.clone(),
            x: 0.0,
            y: ascent,
            font_size: effective_size,
        }],
        lines: Vec::new(),
        paths: Vec::new(),
    })
}

fn layout_delimited(
    left: &Delimiter,
    inner: &AstNode,
    right: &Delimiter,
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    let inner_box = layout_node(inner, font_size, font)?;
    let LayoutBox {
        width: inner_width,
        height: inner_height,
        baseline: inner_baseline,
        script_policy: inner_policy,
        italic_correction: inner_italic,
        items: inner_items,
        lines: inner_lines,
        paths: inner_paths,
    } = inner_box;

    let mut max_above = inner_baseline;
    let mut max_below = inner_height - inner_baseline;

    let left_box = left
        .glyph
        .as_ref()
        .map(|glyph| make_delimiter_box(glyph, inner_height, font_size, font));
    if let Some(ref lb) = left_box {
        max_above = max_above.max(lb.baseline);
        max_below = max_below.max(lb.height - lb.baseline);
    }

    let right_box = right
        .glyph
        .as_ref()
        .map(|glyph| make_delimiter_box(glyph, inner_height, font_size, font));
    if let Some(ref rb) = right_box {
        max_above = max_above.max(rb.baseline);
        max_below = max_below.max(rb.height - rb.baseline);
    }
    let right_italic = right_box
        .as_ref()
        .map(|rb| rb.italic_correction)
        .unwrap_or(inner_italic);

    let baseline = max_above;
    let total_height = max_above + max_below;
    let gap = font_size * 0.12;

    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut paths: Vec<RenderPath> = Vec::new();
    let mut cursor_x = 0.0f32;

    if let Some(lb) = left_box {
        items.extend(offset_items_owned(
            lb.items,
            cursor_x,
            baseline - lb.baseline,
        ));
        lines.extend(offset_lines_owned(
            lb.lines,
            cursor_x,
            baseline - lb.baseline,
        ));
        paths.extend(offset_paths_owned(
            lb.paths,
            cursor_x,
            baseline - lb.baseline,
        ));
        cursor_x += lb.width + gap;
    }

    items.extend(offset_items_owned(
        inner_items,
        cursor_x,
        baseline - inner_baseline,
    ));
    lines.extend(offset_lines_owned(
        inner_lines,
        cursor_x,
        baseline - inner_baseline,
    ));
    paths.extend(offset_paths_owned(
        inner_paths,
        cursor_x,
        baseline - inner_baseline,
    ));
    cursor_x += inner_width;

    if right_box.is_some() {
        cursor_x += gap;
    }

    if let Some(rb) = right_box {
        items.extend(offset_items_owned(
            rb.items,
            cursor_x,
            baseline - rb.baseline,
        ));
        lines.extend(offset_lines_owned(
            rb.lines,
            cursor_x,
            baseline - rb.baseline,
        ));
        paths.extend(offset_paths_owned(
            rb.paths,
            cursor_x,
            baseline - rb.baseline,
        ));
        cursor_x += rb.width;
    }

    Ok(LayoutBox {
        width: cursor_x,
        height: total_height,
        baseline,
        script_policy: inner_policy,
        italic_correction: right_italic,
        items,
        lines,
        paths,
    })
}

fn layout_group(
    children: &[AstNode],
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    if children.is_empty() {
        return layout_text("", font_size, font);
    }
    let mut entries = Vec::with_capacity(children.len());
    let mut cursor_x = 0.0f32;
    let spacing = font_size * 0.1;

    let mut max_above = 0.0f32;
    let mut max_below = 0.0f32;

    for (index, child) in children.iter().enumerate() {
        let child_box = layout_node(child, font_size, font)?;
        let offset_x = if index == 0 { 0.0 } else { spacing };
        cursor_x += offset_x;
        max_above = max_above.max(child_box.baseline);
        max_below = max_below.max(child_box.height - child_box.baseline);
        entries.push((child_box, cursor_x));
        cursor_x += entries.last().unwrap().0.width;
    }

    let baseline = max_above;
    let height = max_above + max_below;
    let width = cursor_x;

    let mut items = Vec::with_capacity(children.len());
    let mut lines = Vec::with_capacity(children.len());
    let mut paths = Vec::with_capacity(children.len());
    let mut trailing_italic = 0.0f32;
    for (child_box, x) in entries {
        trailing_italic = child_box.italic_correction;
        items.extend(offset_items_owned(
            child_box.items,
            x,
            baseline - child_box.baseline,
        ));
        lines.extend(offset_lines_owned(
            child_box.lines,
            x,
            baseline - child_box.baseline,
        ));
        paths.extend(offset_paths_owned(
            child_box.paths,
            x,
            baseline - child_box.baseline,
        ));
    }

    Ok(LayoutBox {
        width,
        height,
        baseline,
        script_policy: ScriptPolicy::Right,
        italic_correction: trailing_italic,
        items,
        lines,
        paths,
    })
}

fn layout_matrix(
    rows: &[Vec<AstNode>],
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    if rows.is_empty() {
        return layout_text("", font_size, font);
    }
    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return layout_text("", font_size, font);
    }

    let mut cell_boxes: Vec<Vec<LayoutBox>> = Vec::with_capacity(rows.len());
    for row in rows {
        let mut row_boxes = Vec::with_capacity(row.len());
        for cell in row {
            row_boxes.push(layout_node(cell, font_size, font)?);
        }
        cell_boxes.push(row_boxes);
    }

    let mut column_widths = vec![0.0f32; col_count];
    for row_boxes in &cell_boxes {
        for col_idx in 0..col_count {
            if let Some(cell) = row_boxes.get(col_idx) {
                column_widths[col_idx] = column_widths[col_idx].max(cell.width);
            }
        }
    }

    let col_gap = font_size * 0.4;
    let row_gap = font_size * 0.35;
    let cell_padding = font_size * 0.25;

    let mut row_metrics = Vec::with_capacity(cell_boxes.len());
    for row_boxes in &cell_boxes {
        let mut max_above = 0.0f32;
        let mut max_below = 0.0f32;
        for cell in row_boxes {
            max_above = max_above.max(cell.baseline);
            max_below = max_below.max(cell.height - cell.baseline);
        }
        row_metrics.push((max_above, max_below));
    }

    let mut col_offsets = Vec::with_capacity(col_count);
    let mut cursor_x = 0.0f32;
    for (idx, width) in column_widths.iter().enumerate() {
        col_offsets.push(cursor_x);
        cursor_x += *width;
        if idx + 1 < col_count {
            cursor_x += col_gap;
        }
    }
    let inner_width = cursor_x;

    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut paths = Vec::new();

    let mut cursor_y = 0.0f32;
    for (row_idx, row_boxes) in cell_boxes.iter().enumerate() {
        let (above, below) = row_metrics[row_idx];
        let row_baseline = above;
        let row_height = above + below;

        for col_idx in 0..col_count {
            if let Some(cell) = row_boxes.get(col_idx) {
                let offset_x = col_offsets[col_idx] + (column_widths[col_idx] - cell.width) / 2.0;
                let offset_y = cursor_y + row_baseline - cell.baseline;
                items.extend(offset_items_owned(cell.items.clone(), offset_x, offset_y));
                lines.extend(offset_lines_owned(cell.lines.clone(), offset_x, offset_y));
                paths.extend(offset_paths_owned(cell.paths.clone(), offset_x, offset_y));
            }
        }

        cursor_y += row_height;
        if row_idx + 1 < cell_boxes.len() {
            cursor_y += row_gap;
        }
    }

    let content_height = cursor_y;
    let total_height = content_height + cell_padding * 2.0;
    let hook_length = font_size * 0.35;
    let bracket_stroke = (font_size * 0.06).max(1.0);
    let side_padding = hook_length + bracket_stroke;
    let total_width = inner_width + side_padding * 2.0;

    offset_items(&mut items, side_padding, cell_padding);
    offset_lines(&mut lines, side_padding, cell_padding);
    offset_paths(&mut paths, side_padding, cell_padding);

    // 绘制矩阵括号线条
    let left_x = bracket_stroke / 2.0;
    let right_x = total_width - bracket_stroke / 2.0;
    let top_y = 0.0;
    let bottom_y = total_height;

    lines.push(RenderLine {
        x1: left_x,
        y1: top_y,
        x2: left_x,
        y2: bottom_y,
        stroke_width: bracket_stroke,
    });
    lines.push(RenderLine {
        x1: left_x,
        y1: top_y,
        x2: left_x + hook_length,
        y2: top_y,
        stroke_width: bracket_stroke,
    });
    lines.push(RenderLine {
        x1: left_x,
        y1: bottom_y,
        x2: left_x + hook_length,
        y2: bottom_y,
        stroke_width: bracket_stroke,
    });

    lines.push(RenderLine {
        x1: right_x,
        y1: top_y,
        x2: right_x,
        y2: bottom_y,
        stroke_width: bracket_stroke,
    });
    lines.push(RenderLine {
        x1: right_x,
        y1: top_y,
        x2: right_x - hook_length,
        y2: top_y,
        stroke_width: bracket_stroke,
    });
    lines.push(RenderLine {
        x1: right_x,
        y1: bottom_y,
        x2: right_x - hook_length,
        y2: bottom_y,
        stroke_width: bracket_stroke,
    });

    let baseline = cell_padding + content_height / 2.0;

    Ok(LayoutBox {
        width: total_width,
        height: total_height,
        baseline,
        script_policy: ScriptPolicy::Right,
        italic_correction: 0.0,
        items,
        lines,
        paths,
    })
}

fn layout_fraction(
    numerator: &AstNode,
    denominator: &AstNode,
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    let num_box = layout_node(numerator, font_size, font)?;
    let den_box = layout_node(denominator, font_size, font)?;

    let padding = font_size * 0.25;
    let gap = font_size * 0.2;
    let line_thickness = (font_size * 0.07).max(1.0);

    let inner_width = num_box.width.max(den_box.width);
    let total_width = inner_width + padding * 2.0;

    let numerator_dx = padding + (inner_width - num_box.width) / 2.0;
    let denominator_dx = padding + (inner_width - den_box.width) / 2.0;

    let numerator_top = padding;
    let line_y = numerator_top + num_box.height + gap;
    let denominator_top = line_y + line_thickness + gap;
    let denominator_baseline_y = denominator_top + den_box.baseline;

    let total_height = denominator_top + den_box.height + padding;
    let baseline = denominator_baseline_y;

    let mut items = Vec::new();
    items.extend(offset_items_owned(
        num_box.items,
        numerator_dx,
        numerator_top,
    ));
    items.extend(offset_items_owned(
        den_box.items,
        denominator_dx,
        denominator_top,
    ));

    let mut lines = Vec::new();
    lines.extend(offset_lines_owned(
        num_box.lines,
        numerator_dx,
        numerator_top,
    ));
    lines.extend(offset_lines_owned(
        den_box.lines,
        denominator_dx,
        denominator_top,
    ));
    let mut paths = Vec::new();
    paths.extend(offset_paths_owned(
        num_box.paths,
        numerator_dx,
        numerator_top,
    ));
    paths.extend(offset_paths_owned(
        den_box.paths,
        denominator_dx,
        denominator_top,
    ));
    lines.push(RenderLine {
        x1: padding,
        y1: line_y + line_thickness / 2.0,
        x2: total_width - padding,
        y2: line_y + line_thickness / 2.0,
        stroke_width: line_thickness,
    });

    Ok(LayoutBox {
        width: total_width,
        height: total_height,
        baseline,
        script_policy: ScriptPolicy::Right,
        italic_correction: 0.0,
        items,
        lines,
        paths,
    })
}

fn layout_sqrt(value: &AstNode, font_size: f32, font: &Font) -> Result<LayoutBox, RenderError> {
    let inner_box = layout_node(value, font_size, font)?;
    let padding = font_size * 0.15;
    let symbol_width = font_size * 0.6;
    let line_thickness = (font_size * 0.06).max(0.8);

    let baseline = padding + inner_box.baseline;
    let total_height = padding * 2.0 + inner_box.height.max(font_size * 1.1);
    let total_width = symbol_width + inner_box.width + padding;

    let mut items = Vec::new();
    items.push(RenderItem {
        text: "√".into(),
        x: 0.0,
        y: baseline,
        font_size: font_size * 1.05,
    });
    items.extend(offset_items_owned(inner_box.items, symbol_width, padding));

    let mut lines = offset_lines_owned(inner_box.lines, symbol_width, padding);
    let paths = offset_paths_owned(inner_box.paths, symbol_width, padding);
    let bar_y = padding + line_thickness;
    lines.push(RenderLine {
        x1: symbol_width,
        y1: bar_y,
        x2: total_width,
        y2: bar_y,
        stroke_width: line_thickness,
    });

    Ok(LayoutBox {
        width: total_width,
        height: total_height,
        baseline,
        script_policy: ScriptPolicy::Right,
        italic_correction: 0.0,
        items,
        lines,
        paths,
    })
}

fn layout_scripts(
    base: &AstNode,
    superscript: Option<&AstNode>,
    subscript: Option<&AstNode>,
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    let base_box = layout_node(base, font_size, font)?;
    let script_font_size = font_size * 0.7;

    let sup_box = match superscript {
        Some(node) => Some(layout_node(node, script_font_size, font)?),
        None => None,
    };
    let sub_box = match subscript {
        Some(node) => Some(layout_node(node, script_font_size, font)?),
        None => None,
    };

    let rendered = match base_box.script_policy {
        ScriptPolicy::Right => layout_scripts_right(base_box, sup_box, sub_box, font_size),
        ScriptPolicy::AboveBelow => layout_scripts_vertical(base_box, sup_box, sub_box, font_size),
    };

    Ok(rendered)
}

fn layout_decorated(
    base: &AstNode,
    decoration: DecorationKind,
    font_size: f32,
    font: &Font,
) -> Result<LayoutBox, RenderError> {
    let base_box = layout_node(base, font_size, font)?;
    let LayoutBox {
        width: base_width,
        height: base_height,
        baseline: base_baseline,
        script_policy: base_policy,
        italic_correction: base_italic,
        items: base_items,
        lines: base_lines,
        paths: base_paths,
    } = base_box;
    let line_thickness = (font_size * 0.05).max(0.8);

    let (padding_top, padding_bottom) = match decoration {
        DecorationKind::Overline
        | DecorationKind::Bar
        | DecorationKind::Hat
        | DecorationKind::Tilde
        | DecorationKind::Vector
        | DecorationKind::Dot
        | DecorationKind::Ddot
        | DecorationKind::Overbrace => (font_size * 0.25, font_size * 0.05),
        DecorationKind::Underline | DecorationKind::Underbrace => {
            (font_size * 0.05, font_size * 0.25)
        }
    };

    let baseline = padding_top + base_baseline;
    let height = padding_top + base_height + padding_bottom;

    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut paths = Vec::new();

    items.extend(offset_items_owned(base_items, 0.0, padding_top));
    lines.extend(offset_lines_owned(base_lines, 0.0, padding_top));
    paths.extend(offset_paths_owned(base_paths, 0.0, padding_top));

    match decoration {
        DecorationKind::Overline | DecorationKind::Bar | DecorationKind::Overbrace => {
            let y = (padding_top - line_thickness / 2.0).max(line_thickness / 2.0);
            lines.push(RenderLine {
                x1: 0.0,
                y1: y,
                x2: base_width,
                y2: y,
                stroke_width: line_thickness,
            });
        }
        DecorationKind::Underline | DecorationKind::Underbrace => {
            let y = padding_top + base_height + line_thickness / 2.0;
            lines.push(RenderLine {
                x1: 0.0,
                y1: y,
                x2: base_width,
                y2: y,
                stroke_width: line_thickness,
            });
        }
        DecorationKind::Hat => {
            let hat_font_size = font_size * 0.7;
            let (hat_ascent, _, _) = line_metrics(font, hat_font_size);
            let hat_width = measure_text_width("^", hat_font_size, font);
            let hat_x = (base_width - hat_width) / 2.0;
            let hat_y = (padding_top * 0.6).max(hat_ascent);
            items.push(RenderItem {
                text: "^".into(),
                x: hat_x,
                y: hat_y,
                font_size: hat_font_size,
            });
        }
        DecorationKind::Tilde => {
            let tilde_font_size = font_size * 0.7;
            let (tilde_ascent, _, _) = line_metrics(font, tilde_font_size);
            let tilde_width = measure_text_width("~", tilde_font_size, font);
            let tilde_x = (base_width - tilde_width) / 2.0;
            let tilde_y = (padding_top * 0.6).max(tilde_ascent);
            items.push(RenderItem {
                text: "~".into(),
                x: tilde_x,
                y: tilde_y,
                font_size: tilde_font_size,
            });
        }
        DecorationKind::Vector => {
            let arrow_font_size = font_size * 0.7;
            let (arrow_ascent, _, _) = line_metrics(font, arrow_font_size);
            let arrow_text = "→";
            let arrow_width = measure_text_width(arrow_text, arrow_font_size, font);
            let arrow_x = (base_width - arrow_width) / 2.0;
            let arrow_y = (padding_top * 0.6).max(arrow_ascent);
            items.push(RenderItem {
                text: arrow_text.into(),
                x: arrow_x,
                y: arrow_y,
                font_size: arrow_font_size,
            });
        }
        DecorationKind::Dot => {
            let dot_font_size = font_size * 0.6;
            let (dot_ascent, _, _) = line_metrics(font, dot_font_size);
            let dot_width = measure_text_width("·", dot_font_size, font);
            let dot_x = (base_width - dot_width) / 2.0;
            let dot_y = (padding_top * 0.5).max(dot_ascent);
            items.push(RenderItem {
                text: "·".into(),
                x: dot_x,
                y: dot_y,
                font_size: dot_font_size,
            });
        }
        DecorationKind::Ddot => {
            let dot_font_size = font_size * 0.55;
            let (dot_ascent, _, _) = line_metrics(font, dot_font_size);
            let dot_width = measure_text_width("·", dot_font_size, font);
            let spacing = dot_width.max(font_size * 0.2);
            let center = base_width / 2.0;
            let dot_y = (padding_top * 0.5).max(dot_ascent);
            let left_x = center - spacing / 2.0 - dot_width / 2.0;
            let right_x = center + spacing / 2.0 - dot_width / 2.0;
            items.push(RenderItem {
                text: "·".into(),
                x: left_x,
                y: dot_y,
                font_size: dot_font_size,
            });
            items.push(RenderItem {
                text: "·".into(),
                x: right_x,
                y: dot_y,
                font_size: dot_font_size,
            });
        }
    }

    Ok(LayoutBox {
        width: base_width,
        height,
        baseline,
        script_policy: base_policy,
        italic_correction: base_italic,
        items,
        lines,
        paths,
    })
}

fn layout_scripts_right(
    base_box: LayoutBox,
    mut sup_box: Option<LayoutBox>,
    mut sub_box: Option<LayoutBox>,
    font_size: f32,
) -> LayoutBox {
    let spacing = font_size * 0.08;
    let sup_raise = font_size * 0.75;
    let sub_drop = font_size * 0.35;

    let LayoutBox {
        width: base_width,
        height: base_height,
        baseline: base_baseline,
        italic_correction: base_italic,
        items: base_items,
        lines: base_lines,
        paths: base_paths,
        ..
    } = base_box;

    let mut above = base_baseline;
    let mut below = base_height - base_baseline;

    if let Some(ref sup) = sup_box {
        above = above.max(sup_raise + sup.height);
    }
    if let Some(ref sub) = sub_box {
        below = below.max(sub_drop + sub.height);
    }

    let baseline = above;
    let height = above + below;

    let sup_width = sup_box.as_ref().map(|b| b.width).unwrap_or(0.0);
    let sub_width = sub_box.as_ref().map(|b| b.width).unwrap_or(0.0);
    let scripts_width = sup_width.max(sub_width);
    let anchor_x = (base_width - base_italic).max(0.0);
    let total_width = if scripts_width > 0.0 {
        anchor_x + spacing + scripts_width
    } else {
        base_width
    };

    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut paths = Vec::new();

    items.extend(offset_items_owned(
        base_items,
        0.0,
        baseline - base_baseline,
    ));
    lines.extend(offset_lines_owned(
        base_lines,
        0.0,
        baseline - base_baseline,
    ));
    paths.extend(offset_paths_owned(
        base_paths,
        0.0,
        baseline - base_baseline,
    ));

    if let Some(sup) = sup_box.take() {
        let dx = anchor_x + spacing;
        let dy = baseline - sup_raise - sup.baseline;
        items.extend(offset_items_owned(sup.items, dx, dy));
        lines.extend(offset_lines_owned(sup.lines, dx, dy));
        paths.extend(offset_paths_owned(sup.paths, dx, dy));
    }

    if let Some(sub) = sub_box.take() {
        let dx = anchor_x + spacing;
        let dy = baseline + sub_drop - sub.baseline;
        items.extend(offset_items_owned(sub.items, dx, dy));
        lines.extend(offset_lines_owned(sub.lines, dx, dy));
        paths.extend(offset_paths_owned(sub.paths, dx, dy));
    }

    LayoutBox {
        width: total_width,
        height,
        baseline,
        script_policy: ScriptPolicy::Right,
        italic_correction: base_italic,
        items,
        lines,
        paths,
    }
}

fn layout_scripts_vertical(
    base_box: LayoutBox,
    mut sup_box: Option<LayoutBox>,
    mut sub_box: Option<LayoutBox>,
    font_size: f32,
) -> LayoutBox {
    let sup_gap = if sup_box.is_some() {
        font_size * 0.2
    } else {
        0.0
    };
    let sub_gap = if sub_box.is_some() {
        font_size * 0.2
    } else {
        0.0
    };

    let LayoutBox {
        width: base_width,
        height: base_height,
        baseline: base_baseline,
        script_policy: base_policy,
        italic_correction: base_italic,
        items: base_items,
        lines: base_lines,
        paths: base_paths,
        ..
    } = base_box;

    let sup_height = sup_box.as_ref().map(|b| b.height).unwrap_or(0.0);

    let total_width = base_width
        .max(sup_box.as_ref().map(|b| b.width).unwrap_or(0.0))
        .max(sub_box.as_ref().map(|b| b.width).unwrap_or(0.0));

    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut paths = Vec::new();
    let mut current_y = 0.0f32;

    if let Some(sup) = sup_box.take() {
        let dx = (total_width - sup.width).max(0.0) / 2.0;
        items.extend(offset_items_owned(sup.items, dx, current_y));
        lines.extend(offset_lines_owned(sup.lines, dx, current_y));
        paths.extend(offset_paths_owned(sup.paths, dx, current_y));
        current_y += sup.height + sup_gap;
    }

    let base_dx = (total_width - base_width).max(0.0) / 2.0;
    let base_offset = current_y;
    items.extend(offset_items_owned(base_items, base_dx, base_offset));
    lines.extend(offset_lines_owned(base_lines, base_dx, base_offset));
    paths.extend(offset_paths_owned(base_paths, base_dx, base_offset));
    current_y += base_height;

    if let Some(sub) = sub_box.take() {
        current_y += sub_gap;
        let dx = (total_width - sub.width).max(0.0) / 2.0;
        items.extend(offset_items_owned(sub.items, dx, current_y));
        lines.extend(offset_lines_owned(sub.lines, dx, current_y));
        paths.extend(offset_paths_owned(sub.paths, dx, current_y));
        current_y += sub.height;
    }

    let height = current_y;
    let baseline = sup_height + sup_gap + base_baseline;

    LayoutBox {
        width: total_width,
        height,
        baseline,
        script_policy: base_policy,
        italic_correction: base_italic,
        items,
        lines,
        paths,
    }
}

/// 读取字体的行距信息，若字体未提供则退化为经验值
fn line_metrics(font: &Font, font_size: f32) -> (f32, f32, f32) {
    if let Some(metrics) = font.horizontal_line_metrics(font_size) {
        let ascent = metrics.ascent.max(0.0);
        let descent = metrics.descent.abs();
        let line_gap = metrics.line_gap.max(0.0);
        (ascent, descent, line_gap)
    } else {
        (
            font_size * 0.8, // 经验值：上方高度约为字号 80%
            font_size * 0.2, // 下方高度约为字号 20%
            0.0,
        )
    }
}

fn make_delimiter_box(
    glyph: &str,
    target_height: f32,
    base_font_size: f32,
    font: &Font,
) -> LayoutBox {
    let (base_ascent, base_descent, _) = line_metrics(font, base_font_size);
    let base_height = base_ascent + base_descent;
    let scale = if target_height <= base_height {
        1.0
    } else {
        (target_height / base_height).min(3.0)
    };
    let effective_size = base_font_size * scale;
    let (ascent, descent, _) = line_metrics(font, effective_size);
    let mut width = 0.0f32;
    let mut italic_correction = 0.0f32;
    for ch in glyph.chars() {
        let metrics = cached_metrics(font, ch, effective_size);
        width += metrics.advance_width;
        italic_correction = glyph_italic_correction(&metrics);
    }
    LayoutBox {
        width,
        height: ascent + descent,
        baseline: ascent,
        script_policy: ScriptPolicy::Right,
        italic_correction,
        items: vec![RenderItem {
            text: glyph.to_string(),
            x: 0.0,
            y: ascent,
            font_size: effective_size,
        }],
        lines: Vec::new(),
        paths: Vec::new(),
    }
}

fn measure_text_width(content: &str, font_size: f32, font: &Font) -> f32 {
    content
        .chars()
        .map(|ch| cached_metrics(font, ch, font_size).advance_width)
        .sum()
}

fn glyph_italic_correction(metrics: &GlyphMetrics) -> f32 {
    if metrics.bounds.width <= 0.0 {
        return 0.0;
    }
    let right_edge = metrics.bounds.xmin + metrics.bounds.width;
    let correction = metrics.advance_width - right_edge;
    if correction > 0.0 {
        correction
    } else {
        0.0
    }
}

fn offset_items(items: &mut [RenderItem], dx: f32, dy: f32) {
    for item in items {
        item.x += dx;
        item.y += dy;
    }
}

fn offset_lines(lines: &mut [RenderLine], dx: f32, dy: f32) {
    for line in lines {
        line.x1 += dx;
        line.x2 += dx;
        line.y1 += dy;
        line.y2 += dy;
    }
}

fn offset_paths(paths: &mut [RenderPath], dx: f32, dy: f32) {
    for path in paths {
        path.x += dx;
        path.y += dy;
    }
}

fn offset_items_owned(items: Vec<RenderItem>, dx: f32, dy: f32) -> Vec<RenderItem> {
    let mut moved = items;
    offset_items(&mut moved, dx, dy);
    moved
}

fn offset_lines_owned(lines: Vec<RenderLine>, dx: f32, dy: f32) -> Vec<RenderLine> {
    let mut moved = lines;
    offset_lines(&mut moved, dx, dy);
    moved
}

fn offset_paths_owned(paths: Vec<RenderPath>, dx: f32, dy: f32) -> Vec<RenderPath> {
    let mut moved = paths;
    offset_paths(&mut moved, dx, dy);
    moved
}

fn cached_metrics(font: &Font, ch: char, font_size: f32) -> GlyphMetrics {
    let quantized = (font_size * 100.0).round() as u32;
    METRICS_CACHE.with(|cache| {
        if let Some(metrics) = cache.borrow().get(&(ch, quantized)) {
            return *metrics;
        }
        let metrics = font.metrics(ch, font_size);
        cache.borrow_mut().insert((ch, quantized), metrics);
        metrics
    })
}

thread_local! {
    static METRICS_CACHE: RefCell<HashMap<(char, u32), GlyphMetrics>> =
        RefCell::new(HashMap::new());
}
