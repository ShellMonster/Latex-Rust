use std::path::PathBuf;
use std::{env, fs};

use sha2::{Digest, Sha256};

const DEFAULT_FORMULA: &str = r"P_{mediaBidPrice} = \min\left(\max\left(P_{channelSettlePrice} \times \left(1 - \alpha \cdot \frac{P_{channelSettlePrice} - P_{midPrice}}{P_{channelSettlePrice} + P_{midPrice}}\right), \min\left(P_{mediaBidFloor} 0.01, \max(P_{channelSettlePrice}, P_{mediaBidFloor})\right)\right), P_{channelSettlePrice}\right)";

fn main() {
    let formula = env::var("FORMULA")
        .ok()
        .or_else(|| env::args().nth(1))
        .unwrap_or_else(|| DEFAULT_FORMULA.to_string());

    let mut hasher = Sha256::new();
    hasher.update(formula.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let mut output_path = PathBuf::from("output_svg");
    output_path.push(format!("{}.svg", hash));

    if output_path.exists() {
        println!("文件已存在，无需重复生成: {:?}", output_path);
        return;
    }

    let start = std::time::Instant::now();
    match formula_render::render_formula(&formula) {
        Ok(svg) => {
            let elapsed = start.elapsed();
            if let Err(err) = fs::write(&output_path, svg) {
                eprintln!("写入 SVG 失败: {}", err);
            } else {
                println!("已生成 SVG: {:?}，耗时: {:.3?}", output_path, elapsed);
            }
        }
        Err(err) => eprintln!("渲染失败: {}", err),
    }
}
