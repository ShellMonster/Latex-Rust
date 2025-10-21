#!/usr/bin/env bash

# 这个脚本用来快速编译 Rust 渲染引擎并生成共享库文件
set -euo pipefail

# 切换到脚本所在目录，确保 cargo 在正确位置执行
cd "$(dirname "$0")"

# 构建 release 版本，输出更小更快
cargo build --release

# 把不同平台的共享库复制到根目录，方便 Go 直接调用
TARGET_DIR="target/release"
OUTPUT_DIR="."

cp "${TARGET_DIR}/libformula_render.dylib" "${OUTPUT_DIR}/libformula.dylib" 2>/dev/null || true
cp "${TARGET_DIR}/libformula_render.so" "${OUTPUT_DIR}/libformula.so" 2>/dev/null || true
cp "${TARGET_DIR}/formula_render.dll" "${OUTPUT_DIR}/formula.dll" 2>/dev/null || true
