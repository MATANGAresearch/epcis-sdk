use criterion::{Criterion, criterion_group, criterion_main};
use epcis_translate::{Sgtin, Sscc};
use std::hint::black_box;

fn bench_translate(c: &mut Criterion) {
    let sgtin_urn = "urn:epc:id:sgtin:4012345.098765.12345";
    let sgtin_dl = "https://id.gs1.org/01/04012345987652/21/12345";

    let sscc_urn = "urn:epc:id:sscc:4012345.0123456789";
    let sscc_dl = "https://id.gs1.org/00/340123450123456785";

    c.bench_function("SGTIN URN to DL", |b| {
        b.iter(|| {
            let sgtin = Sgtin::from_urn(black_box(sgtin_urn)).unwrap();
            let _ = black_box(sgtin.to_digital_link("https://id.gs1.org"));
        })
    });

    c.bench_function("SGTIN DL to URN", |b| {
        b.iter(|| {
            let sgtin = Sgtin::from_digital_link(black_box(sgtin_dl), 7).unwrap();
            let _ = black_box(sgtin.to_urn());
        })
    });

    c.bench_function("SSCC URN to DL", |b| {
        b.iter(|| {
            let sscc = Sscc::from_urn(black_box(sscc_urn)).unwrap();
            let _ = black_box(sscc.to_digital_link("https://id.gs1.org"));
        })
    });

    c.bench_function("SSCC DL to URN", |b| {
        b.iter(|| {
            let sscc = Sscc::from_digital_link(black_box(sscc_dl), 7).unwrap();
            let _ = black_box(sscc.to_urn());
        })
    });
}

criterion_group!(benches, bench_translate);
criterion_main!(benches);
