#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// 模块入口：对外提供渲染接口，并串联各子模块

use std::borrow::Cow;
use std::panic::{catch_unwind, AssertUnwindSafe}; // 引入 panic 捕获工具，防止单次渲染拖垮进程

use rayon::prelude::*; // 引入 rayon 并行迭代器，后面批量渲染会用到

mod ast; // 语法树定义模块
mod config; // 运行时配置管理
mod error; // 错误类型模块，统一描述错误信息
mod ffi; // FFI 模块，提供 C 可调用的接口
mod init; // 初始化模块，加载字体与全局状态
mod layout; // 排版模块，把语法树转换为布局信息
mod parse; // 解析模块，把 LaTeX 字符串解析成语法树
mod render; // 渲染模块，把布局信息转成 SVG 字符串

pub use crate::error::RenderError; // 暴露错误类型，方便调用方处理
pub use crate::ffi::{free_svg, render_svg}; // 暴露 C 接口，让 Go 通过 cgo 调用并负责释放内存
pub use config::{override_svg_text_mode, SvgTextMode}; // 提供外部调整 SVG 输出模式的入口（可选使用）

/// 对外提供的核心函数：输入 LaTeX，输出 SVG
pub fn render_formula(tex: &str) -> Result<String, RenderError> {
    init::ensure_fonts_loaded()?; // 确保字体与全局状态已经就绪，失败直接返回错误
    let trimmed = tex.trim(); // 去掉首尾空白，避免无意义字符影响结果
    if trimmed.is_empty() {
        // 如果内容为空，直接返回自定义错误
        return Err(RenderError::EmptyInput); // 提示调用方输入为空
    }

    let normalized = normalize_escaped_commands(trimmed);

    let guarded_result = catch_unwind(AssertUnwindSafe(|| {
        // 用 catch_unwind 捕获潜在 panic
        parse::parse(normalized.as_ref()) // 第一步：解析得到语法树
            .and_then(|ast| layout::layout(&ast)) // 第二步：根据语法树生成布局数据
            .and_then(|layout| render::render_svg_document(&layout)) // 第三步：把布局转成 SVG 字符串
    }));

    let svg = match guarded_result {
        // 统一处理 catch_unwind 与中间错误
        Ok(Ok(svg)) => svg,              // 正常情况：成功得到 SVG
        Ok(Err(err)) => return Err(err), // 解析或渲染阶段返回业务错误
        Err(_) => return Err(RenderError::UnexpectedPanic), // 捕获 panic，转换成安全的错误提示
    };

    Ok(svg) // 返回最终 SVG 字符串
}

/// 批量渲染接口：给 rayon 使用，提升并发性能
pub fn render_formula_batch(texts: &[String]) -> Vec<Result<String, RenderError>> {
    texts
        .par_iter() // 开启 rayon 并行迭代
        .map(|tex| render_formula(tex)) // 对每个字符串调用单次渲染逻辑
        .collect() // 把结果收集成 Vec
}

#[cfg(test)] // 仅在测试环境编译下面的代码
mod tests {
    use super::*; // 把父模块公开项目引入作用域
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static MODE_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test] // 声明一个单元测试
    fn simple_formula_should_render() {
        let _guard = MODE_GUARD.lock().unwrap();
        override_svg_text_mode(Some(SvgTextMode::Text));
        let svg = render_formula("a + b").expect("渲染失败");
        assert!(
            svg.contains("<text"),
            "文本模式下应该保留 <text> 节点，当前输出: {svg}"
        );
        override_svg_text_mode(None);
    }

    #[test] // 声明第二个单元测试
    fn batch_render_should_work() {
        let _guard = MODE_GUARD.lock().unwrap();
        override_svg_text_mode(Some(SvgTextMode::Text));
        // 测试批量渲染流程
        let inputs = vec!["E=mc^2".to_string(), "\\frac{1}{2}".to_string()]; // 准备两个公式
        let outputs = render_formula_batch(&inputs); // 调用批量渲染
        assert_eq!(outputs.len(), 2, "输出数量要和输入一致"); // 校验数量
        assert!(
            outputs.iter().all(|item| item.is_ok()),
            "所有公式都应该渲染成功"
        ); // 校验全部成功
        override_svg_text_mode(None);
    }

    #[test]
    fn sum_with_scripts_and_matrix_should_render() {
        let _guard = MODE_GUARD.lock().unwrap();
        override_svg_text_mode(Some(SvgTextMode::Text));
        let sum_svg = render_formula("\\sum_{i=1}^{n} i^2").expect("带上下标的求和符号渲染失败"); // 测试大型运算符的上下标布局
        assert!(
            sum_svg.contains('∑'),
            "求和符号应当以文本形式输出，当前内容: {sum_svg}"
        );

        let matrix_svg = render_formula("\\matrix{1 & 2 \\\\ 3 & 4}").expect("矩阵渲染失败"); // 测试矩阵结构排版
        assert!(
            matrix_svg.contains("<line"),
            "矩阵外框应当包含线条，当前输出: {matrix_svg}"
        );
        override_svg_text_mode(None);
    }

    #[test]
    fn can_render_paths_when_requested() {
        let _guard = MODE_GUARD.lock().unwrap();
        override_svg_text_mode(Some(SvgTextMode::Paths));
        let svg = render_formula("x^2 + y^2").expect("路径模式渲染失败");
        assert!(
            svg.contains("<path"),
            "路径模式下应输出 <path> 元素，当前输出: {svg}"
        );
        override_svg_text_mode(None);
    }

    #[test]
    fn extended_symbols_should_render_as_expected() {
        let _guard = MODE_GUARD.lock().unwrap();
        override_svg_text_mode(Some(SvgTextMode::Text));
        let formula = "\\pm \\mp \\leq \\geq \\neq \\rightarrow \\Leftrightarrow \\cdots \\infty \
                       \\forall \\alpha \\Delta \\ell \\emptyset \\hbar";
        let svg = render_formula(formula).expect("扩展符号渲染失败");
        for expected in [
            "±", "∓", "≤", "≥", "≠", "→", "⇔", "⋯", "∞", "∀", "α", "Δ", "ℓ", "∅", "ℏ",
        ] {
            assert!(
                svg.contains(expected),
                "SVG 应包含符号 {expected}，当前输出: {svg}"
            );
        }
        override_svg_text_mode(None);
    }

    #[test]
    fn environments_and_styles_should_render() {
        let _guard = MODE_GUARD.lock().unwrap();
        override_svg_text_mode(Some(SvgTextMode::Text));

        let cases = render_formula("\\begin{cases} x & x > 0 \\\\ -x & x \\leq 0 \\end{cases}")
            .expect("cases 环境渲染失败");
        assert!(
            cases.contains("{"),
            "cases 环境应当包含左花括号，当前输出: {cases}"
        );

        let pmatrix = render_formula("\\begin{pmatrix}1 & 0 \\\\ 0 & 1 \\end{pmatrix}")
            .expect("pmatrix 环境渲染失败");
        assert!(
            pmatrix.contains("(") && pmatrix.contains(")"),
            "pmatrix 应当包含圆括号，当前输出: {pmatrix}"
        );

        let arrow = render_formula("\\xrightarrow{f}").expect("xrightarrow 渲染失败");
        assert!(
            arrow.contains('→') && arrow.contains('f'),
            "扩展箭头应包含箭头与标签，当前输出: {arrow}"
        );

        let tilde = render_formula("\\tilde{x}").expect("tilde 渲染失败");
        assert!(
            tilde.contains('~'),
            "波浪符装饰应添加 '~' 符号，当前输出: {tilde}"
        );

        let bold = render_formula("\\mathbf{AB}").expect("粗体字母渲染失败");
        assert!(
            bold.contains('𝐀') || bold.contains('𝐁'),
            "粗体映射应输出数学粗体字符，当前输出: {bold}"
        );

        override_svg_text_mode(None);
    }
}

fn normalize_escaped_commands(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let mut idx = 0;
    while idx + 2 <= bytes.len() {
        if bytes[idx] == b'\\' && bytes[idx + 1] == b'\\' {
            if idx + 2 < bytes.len() && bytes[idx + 2].is_ascii_alphabetic() {
                let mut output = String::with_capacity(input.len());
                output.push_str(&input[..idx]);
                output.push('\\');
                idx += 2;
                while idx < bytes.len() {
                    let ch = bytes[idx] as char;
                    if ch == '\\' && idx + 1 < bytes.len() && bytes[idx + 1] == b'\\' {
                        if idx + 2 < bytes.len() && bytes[idx + 2].is_ascii_alphabetic() {
                            output.push('\\');
                            idx += 2;
                            continue;
                        }
                    }
                    output.push(ch);
                    idx += 1;
                }
                return Cow::Owned(output);
            }
        }
        idx += 1;
    }
    Cow::Borrowed(input)
}
