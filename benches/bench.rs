extern crate globber;
extern crate criterion;

use criterion::*;
use globber::Pattern;

fn glob_benchmark(c: &mut Criterion) {
    c.bench("matches",
        Benchmark::new("some/**/**/needle.txt", |b| {
            let p = Pattern::new("some/**/**/needle.txt").unwrap();
            b.iter(|| black_box(p.matches("some/one/two/needle.txt")));
        }).throughput(Throughput::Bytes("some/one/two/needle.txt".len() as u32)),
    );
    c.bench("matches",
            Benchmark::new("a*a*a*a*a*a*a*a*a", |b| {
                let p = Pattern::new("a*a*a*a*a*a*a*a*a").unwrap();
                b.iter(|| black_box(p.matches("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")));
            }).throughput(Throughput::Bytes("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".len() as u32)),
    );
    c.bench("matches",
            Benchmark::new("*hello.txt", |b| {
                let p = Pattern::new("*hello.txt").unwrap();
                b.iter(|| black_box(p.matches("gareth_says_hello.txt")));
            }).throughput(Throughput::Bytes("gareth_says_hello.txt".len() as u32)),
    );
    c.bench("matches",
            Benchmark::new("*hello.txt", |b| {
                let p = Pattern::new("*hello.txt").unwrap();
                b.iter(|| black_box(p.matches("gareth_says_hello.txt")));
            }).throughput(Throughput::Bytes("gareth_says_hello.txt".len() as u32)),
    );
}

criterion_group!(benches, glob_benchmark);
criterion_main!(benches);