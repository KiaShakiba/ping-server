use criterion::{criterion_group, criterion_main, Criterion};

mod std_server;
mod tokio_server;

criterion_main! {
    benches
}

criterion_group!(benches, comparison,);

fn comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");
    std_server::bench_std_server(&mut group);
    tokio_server::bench_tokio_server(&mut group);
    group.finish();
}
