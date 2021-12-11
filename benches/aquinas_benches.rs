use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SUPPORTED: &'static [&'static str] = &["mp3", "ogg", "wav", "flac"];

fn supported_match(ext: &str) -> bool {
    match ext {
        "mp3" | "ogg" | "wav" | "flac" => true,
        _ => false,
    }
}
fn supported_contains(ext: &str) -> bool {
    SUPPORTED.contains(&ext)
}

fn structures_benchmark(c: &mut Criterion) {
    c.bench_function("supported match", |b| {
        b.iter(|| supported_match(black_box("wav")))
    });
    c.bench_function("supported contains", |b| {
        b.iter(|| supported_contains(black_box("wav")))
    });
}

criterion_group!(benches, structures_benchmark);
criterion_main!(benches);
