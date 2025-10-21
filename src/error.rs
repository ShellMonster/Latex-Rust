//! 错误类型模块：统一描述渲染过程中可能出现的异常情况

use thiserror::Error; // 引入 thiserror 帮助我们简洁地定义错误枚举

/// 渲染流程中用来传播的错误枚举
#[derive(Debug, Error, Clone)] // 自动实现 Debug、Clone 和 Error 接口，方便调试与复制
pub enum RenderError {
    /// 用户传入了空字符串
    #[error("输入的 LaTeX 公式为空")]
    EmptyInput, // 表示输入为空的错误
    /// LaTeX 解析阶段失败
    #[error("解析 LaTeX 公式失败: {0}")]
    ParseError(String), // 保存解析阶段的详细错误信息
    /// 排版布局阶段失败
    #[error("排版布局失败: {0}")]
    LayoutError(String), // 保存布局阶段的详细错误信息
    /// SVG 输出阶段失败
    #[error("SVG 渲染失败: {0}")]
    RenderFailure(String), // 保存渲染阶段的详细错误信息
    /// 字体文件无法正常加载
    #[error("字体加载失败: {0}")]
    FontLoadError(String), // 保存字体加载相关的信息
    /// 字符串包含非法的 UTF-8 编码
    #[error("公式不是合法的 UTF-8 文本")]
    InvalidUtf8, // 当跨语言传入的字符串编码不正确时使用
    /// 捕获 panic 后返回的通用错误
    #[error("内部渲染发生未知异常")]
    UnexpectedPanic, // 统一 panic 捕获后的错误
}
