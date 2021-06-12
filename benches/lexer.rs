use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use std::str;
use smallstr::SmallString;
use smallvec::{SmallVec, smallvec};

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
struct Record {
    pub label_pairs: Vec<LabelPair>,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
struct LabelPair {
    pub key: Box<str>,
    pub val: Box<str>,
}


struct SmallRecord {
    pub label_pairs: SmallVec<[SmallLabelPair; 16]>,
}

struct SmallLabelPair {
    pub key: SmallString<[u8; 16]>,
    pub val: SmallString<[u8; 32]>,
}

fn benchserde(l: String) -> Record {
    serde_json::from_str(l.as_str()).unwrap()
}


#[inline]
fn find_next(chars: &[u8], start: usize, matcher: u8) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == matcher {
            return Some(i)
        }
    }
    return None
}

#[inline]
fn next_non_space_char(chars: &[u8], start: usize) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] != b' ' {
            return Some(i)
        }
    }
    return None
}

fn parse_record(l: &str) -> Option<Record> {
    let chars = l.as_bytes();
    let mut left_bound = next_non_space_char(chars, 0).unwrap();
    if chars[left_bound] != b'{' {
        return None
    }
    left_bound += 1;

    let mut right_bound: usize;
    let mut label_pairs = Vec::new();
    loop {
        left_bound = next_non_space_char(chars, left_bound).unwrap();
        right_bound = find_next(chars,left_bound, b'=').unwrap();
        let key = &chars[left_bound..right_bound];
        left_bound = find_next(chars,left_bound, b'"').unwrap()+1;
        right_bound = find_next(chars,left_bound, b'"').unwrap();
        let val = &chars[left_bound..right_bound];
        left_bound = right_bound + 1;
        let lp = LabelPair{ key: Box::from(str::from_utf8(key).unwrap().trim_end()), val: Box::from(str::from_utf8(val).unwrap())};
        label_pairs.push(lp);

        left_bound = next_non_space_char(chars, left_bound).unwrap();

        match chars[left_bound] {
            b',' => left_bound += 1,
            b'}' => break,
            _ => return None,
        };
    } 
    return Some(Record{label_pairs})
}

fn parse_small_record(l: &str) -> Option<SmallRecord> {
    let chars = l.as_bytes();
    let mut left_bound = next_non_space_char(chars, 0).unwrap();
    if chars[left_bound] != b'{' {
        return None
    }
    left_bound += 1;

    let mut right_bound: usize;
    let mut label_pairs = SmallVec::new();
    loop {
        left_bound = next_non_space_char(chars, left_bound).unwrap();
        right_bound = find_next(chars,left_bound, b'=').unwrap();
        let key = &chars[left_bound..right_bound];
        left_bound = find_next(chars,left_bound, b'"').unwrap()+1;
        right_bound = find_next(chars,left_bound, b'"').unwrap();
        let val = &chars[left_bound..right_bound];
        left_bound = right_bound + 1;
        let lp = SmallLabelPair{ key: SmallString::from_str(str::from_utf8(key).unwrap().trim_end()), val: SmallString::from_str(str::from_utf8(val).unwrap().trim_end())};
        label_pairs.push(lp);

        left_bound = next_non_space_char(chars, left_bound).unwrap();

        match chars[left_bound] {
            b',' => left_bound += 1,
            b'}' => break,
            _ => return None,
        };
    } 
    return Some(SmallRecord{label_pairs})
}


fn parse_small_record_trim(l: &str) -> Option<SmallRecord> {
    let chars = l.as_bytes();
    let mut left_bound = next_non_space_char(chars, 0).unwrap();
    if chars[left_bound] != b'{' {
        return None
    }
    left_bound += 1;

    let mut right_bound: usize;
    let mut label_pairs = SmallVec::new();
    loop {
        left_bound = next_non_space_char(chars, left_bound).unwrap();
        right_bound = find_next(chars,left_bound, b'=').unwrap();
        let key = &chars[left_bound..right_bound];
        left_bound = find_next(chars,left_bound, b'"').unwrap()+1;
        right_bound = find_next(chars,left_bound, b'"').unwrap();
        let val = &chars[left_bound..right_bound];
        left_bound = right_bound + 1;
        let lp = SmallLabelPair{ key: SmallString::from_str(str::from_utf8(key).unwrap()), val: SmallString::from_str(str::from_utf8(val).unwrap())};
        label_pairs.push(lp);

        left_bound = next_non_space_char(chars, left_bound).unwrap();

        match chars[left_bound] {
            b',' => left_bound += 1,
            b'}' => break,
            _ => return None,
        };
    } 
    return Some(SmallRecord{label_pairs})
}

fn criterion_benchmark(c: &mut Criterion) {
    let field_json = "{\"label_pairs\": [{\"key\": \"author_family_name\", \"val\": \"Daniels\"}, {\"key\": \"author_first_name\", \"val\": \"B\"}, {\"key\": \"author_surname\", \"val\": \"J\"}, {\"key\": \"language\", \"val\": \"English\"}, {\"key\": \"year\", \"val\": \"0\"}, {\"key\": \"extension\", \"val\": \"rar\"}, {\"key\": \"title\", \"val\": \"Stolen Moments\"}, {\"key\": \"publisher\", \"val\": \"\"}, {\"key\": \"edition\", \"val\": \"\"}]}";
    let field_custom = "{author_family_name=\"Daniels\", author_first_name=\"B\",author_surname=\"J\", language=\"English\", year=\"0\", extension=\"rar\", title=\"Stolen Moments\",publisher=\"\",edition=\"\"}";
    c.bench_function("serde mode", |b| {
        b.iter(|| benchserde(black_box(String::from(field_json))))
    });
    c.bench_function("custom mode", |b| {
        b.iter(|| parse_record(black_box(field_custom)))
    });
    c.bench_function("custom smallmode", |b| {
        b.iter(|| parse_small_record(black_box(field_custom)))
    });
    c.bench_function("custom smallmode notrim", |b| {
        b.iter(|| parse_small_record_trim(black_box(field_custom)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
