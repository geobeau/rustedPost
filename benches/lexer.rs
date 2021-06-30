use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_post::lexer;
use rusted_post::record;

fn benchserde(l: String) -> record::Record {
    serde_json::from_str(l.as_str()).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let field_json = "{\"label_pairs\": [{\"key\": \"author_family_name\", \"val\": \"Daniels\"}, {\"key\": \"author_first_name\", \"val\": \"B\"}, {\"key\": \"author_surname\", \"val\": \"J\"}, {\"key\": \"language\", \"val\": \"English\"}, {\"key\": \"year\", \"val\": \"0\"}, {\"key\": \"extension\", \"val\": \"rar\"}, {\"key\": \"title\", \"val\": \"Stolen Moments\"}, {\"key\": \"publisher\", \"val\": \"\"}, {\"key\": \"edition\", \"val\": \"\"}]}";
    let field_custom = "{author_family_name=\"Daniels\", author_first_name=\"B\",author_surname=\"J\", language=\"English\", year=\"0\", extension=\"rar\", title=\"Stolen Moments\",publisher=\"\",edition=\"\"}";
    c.bench_function("serde mode", |b| b.iter(|| benchserde(black_box(String::from(field_json)))));
    c.bench_function("custom (stack) mode", |b| b.iter(|| lexer::parse_small_record(black_box(field_custom))));
    c.bench_function("custom (logos) mode", |b| b.iter(|| lexer::parse_record(black_box(field_custom))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
