use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fsdr_blocks::agc::Agc;
use futuresdr::runtime::mocker::{Mocker, Reader, Writer};
use rand::RngExt;

pub fn agc_f32(c: &mut Criterion) {
    let n_samp = 8192;
    let mut rng = rand::rng();
    let input: Vec<f32> = (0..n_samp).map(|_| rng.random()).collect();

    let mut group = c.benchmark_group("agc");
    group.throughput(Throughput::Elements(n_samp as u64));

    group.bench_function("agc_f32", |b| {
        b.iter(|| {
            let block: Agc<f32, Reader<f32>, Writer<f32>> = Agc::new(0.0, 100.0, 1.0, 0.01, 1.0, false, false);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input.clone());
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, agc_f32);
criterion_main!(benches);
