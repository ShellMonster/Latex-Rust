# Rust 渲染核心

该目录提供数学公式渲染的核心实现：输入 LaTeX 字符串，输出 SVG 文本，可独立运行，也可编译成共享库给 Go 等语言调用。

---

## 项目结构

```
Rust渲染/
├── Cargo.toml                    # Rust 依赖与构建配置
├── build.sh                      # 一键生成共享库脚本
├── fonts/
│   ├── latinmodern-math.otf      # 默认数学字体
│   └── latinmodern-math.woff2    # 可选：字体子集（需自行生成）
├── src/
│   ├── lib.rs                    # 渲染入口：render_formula/render_svg
│   ├── ast/                      # AST 定义（节点、装饰、运算符等）
│   ├── parse/                    # LaTeX 解析（lexer + 规则）
│   ├── layout.rs                 # 排版：脚标、矩阵、装饰、定界符
│   ├── render.rs                 # SVG 输出，支持文本或 usvg 路径模式
│   ├── init.rs                   # 字体懒加载、once_cell
│   └── ffi.rs                    # FFI 接口（render_svg/free_svg）
└── src/bin/render_svg.rs         # 命令行示例，输出 SVG 文件到 output_svg/
```

---

## 快速开始

```bash
cd Rust渲染
cargo check && cargo test            # 编译与单元测试
FORMULA="E=mc^2" cargo run --bin render_svg
# 默认输出文本模式 SVG 到 output_svg/*.svg
```

常用环境变量：

| 变量 | 默认 | 说明 |
| ---- | ---- | ---- |
| `FORMULA` | `render_svg.rs` 内置示例 | 命令行渲染公式 |
| `FORMULA_SVG_MODE` | `text` | `text`：输出 `<text>`；`paths`：运行 usvg/resvg 转换为 `<path>` |
| `FORMULA_SVG_EMBED_FONT` | `0` | `1` 时在 SVG 中嵌入 `@font-face`（体积会增大到数百 KB） |

---

## 编译共享库

```bash
cd Rust渲染
./build.sh
```

脚本会生成：

- macOS: `libformula.dylib`
- Linux: `libformula.so`
- Windows: `formula.dll`

Go 侧 cgo 示例见仓库根目录《Go对接指南.md》。

---

## 性能表现

### Criterion（文本模式）

| 公式 | 中位耗时 |
| ---- | -------- |
| `E=mc^2` | ≈ 1.09 µs |
| `P_{mediaBidPrice} = ...` | ≈ 24.0 µs |

### Go FFI 基准（`go test -bench`）

| 场景 | 平均耗时 |
| ---- | -------- |
| 简单公式顺序 | 917 ns/次 |
| 简单公式并行 | 224 ns/次 |
| 复杂公式顺序 | 9.07 µs/次 |
| 复杂公式并行 | 1.87 µs/次 |

### HTTP 示例（200 QPS 成功样本）

| 指标 | 数值 |
| ---- | ---- |
| 吞吐率 | 200 req/s |
| 平均延迟 | 0.973 ms |
| P95 | 1.465 ms |
| P99 | 3.469 ms |

---

## 设计亮点

- 解析扩展：函数、符号、装饰、矩阵/环境等常见 LaTeX 语法均已覆盖，命令映射使用静态查表（`phf`）。
- 排版优化：斜体校正、脚标垂直布局、装饰箭头/点号/brace、矩阵列宽都在布局阶段完成。
- 性能优化：字形度量线程本地缓存、SVG builder 预估容量、字符串零拷贝转义，使简单公式达到微秒级。
- 输出模式：默认 `<text>` + 字体映射；如需无字体依赖，可切换 usvg/resvg 或开启字体内嵌（体积会增大）。

---

## 参考资料

- 《性能对比报告.md》：Rust 与 Node(Katex) 的对比、复现脚本。
- `Go服务端/性能测试/性能表现报告.md`：端到端压测与 FFI 基准细节。
- `../Go对接指南.md`：共享库编译、cgo 包装和部署建议。

需要进一步定制（如新的 LaTeX 命令、特定样式或批量接口），可在现有架构上扩展相应模块。欢迎在项目根目录的 README 中获取更多整体信息。 -*-
