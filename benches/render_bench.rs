use criterion::{black_box, criterion_group, criterion_main, Criterion}; // 引入 Criterion 基准测试工具

/// 针对核心渲染函数的性能基准，涵盖简单与复杂公式
fn render_formula_benchmark(c: &mut Criterion) {
    let simple = "E=mc^2";
    let complex = r"P_{mediaBidPrice} = \min\left(\max\left(P_{channelSettlePrice} \times \left(1 - \alpha \cdot \frac{P_{channelSettlePrice} - P_{midPrice}}{P_{channelSettlePrice}+P_{midPrice}}\right), \min\left(P_{mediaBidFloor} 0.01, \max(P_{channelSettlePrice}, P_{mediaBidFloor})\right)\right), P_{channelSettlePrice}\right)";

    c.bench_function("render_simple_formula", |b| {
        b.iter(|| {
            let result = formula_render::render_formula(black_box(simple));
            assert!(result.is_ok(), "简单公式渲染应当成功");
        });
    });

    c.bench_function("render_complex_formula", |b| {
        b.iter(|| {
            let result = formula_render::render_formula(black_box(complex));
            assert!(result.is_ok(), "复杂公式渲染应当成功");
        });
    });
}

criterion_group!(benches, render_formula_benchmark);
criterion_main!(benches);
