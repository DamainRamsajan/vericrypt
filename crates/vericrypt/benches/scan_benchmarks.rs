use criterion::{black_box, Criterion};
pub fn bench_scan(c: &mut Criterion) { c.bench_function("scan_empty", |b| { b.iter(|| black_box(0)) }); }
criterion::criterion_group!(benches, bench_scan);
criterion::criterion_main!(benches);
