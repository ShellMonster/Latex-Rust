//! FFI 模块：向 Go 等外部语言暴露 C 兼容接口

use std::ffi::{CStr, CString}; // 引入 C 字符串转换相关类型
use std::os::raw::c_char; // 引入 C 语言字符类型

use crate::error::RenderError; // 引入错误类型，便于做模式匹配
use crate::render_formula; // 引入核心渲染函数

/// 统一定义当渲染失败时返回的兜底 SVG
const INVALID_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="0" y="14" font-size="14" fill="red">Invalid Formula</text></svg>"#; // 简单的错误提示 SVG

/// 将 Rust 字符串转换为 C 字符串指针
fn string_to_c_pointer(text: &str) -> *mut c_char {
    CString::new(text) // 试图创建 C 字符串
        .unwrap_or_else(|_| CString::new("").expect("空字符串一定可以转换")) // 如果包含 \0 则退化为返回空串
        .into_raw() // 把 CString 交给调用者管理
}

/// 根据错误类型选择合适的错误提示 SVG
fn error_to_svg(err: RenderError) -> *mut c_char {
    let message = match err { // 根据不同错误类型决定提示内容
        RenderError::EmptyInput => "<svg xmlns=\"http://www.w3.org/2000/svg\"><text x=\"0\" y=\"14\" font-size=\"14\" fill=\"red\">Empty Formula</text></svg>",
        RenderError::InvalidUtf8 => "<svg xmlns=\"http://www.w3.org/2000/svg\"><text x=\"0\" y=\"14\" font-size=\"14\" fill=\"red\">Invalid UTF-8</text></svg>",
        _ => INVALID_SVG, // 其他错误使用通用提示
    };
    string_to_c_pointer(message) // 返回对应的 SVG
}

/// C 可调用的渲染入口
#[no_mangle] // 确保函数名不被编译器修改
pub extern "C" fn render_svg(tex: *const c_char) -> *mut c_char {
    if tex.is_null() {
        // 判断指针是否为空
        return string_to_c_pointer(INVALID_SVG); // 为空则直接返回错误 SVG
    }

    let input = unsafe { CStr::from_ptr(tex) }; // 将 C 指针视作 CStr
    let formula_str = match input.to_str() {
        // 尝试转换为 UTF-8 字符串
        Ok(content) => content,                            // 成功则获得内容
        Err(_) => return string_to_c_pointer(INVALID_SVG), // 失败则返回错误 SVG
    };

    match render_formula(formula_str) {
        // 调用核心渲染函数
        Ok(svg) => string_to_c_pointer(&svg), // 成功渲染则返回 SVG 字符串
        Err(err) => error_to_svg(err),        // 失败则根据错误类型选择提示
    }
}

/// 供外部语言在使用完字符串后释放内存
#[no_mangle] // 同样确保符号名稳定
pub extern "C" fn free_svg(ptr: *mut c_char) {
    if ptr.is_null() {
        // 避免对空指针重复释放
        return; // 空指针直接返回
    }
    unsafe {
        let _ = CString::from_raw(ptr); // 把指针重新包装成 CString，让 Rust 帮忙释放
    }
}
