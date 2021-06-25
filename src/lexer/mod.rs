use super::record;
use smallstr::SmallString;
use smallvec::SmallVec;
use std::str;

#[inline]
fn next_non_space_char(chars: &[u8], start: usize) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] != b' ' {
            return Some(i);
        }
    }
    return None;
}

#[inline]
fn find_next(chars: &[u8], start: usize, matcher: u8) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == matcher {
            return Some(i);
        }
    }
    return None;
}

#[inline]
fn is_escaped(chars: &[u8], start: usize) -> bool {
    if start == 0 {
        return false;
    }

    chars[start - 1] == b'\\' && !is_escaped(chars, start - 1)
}

#[inline]
// TODO Add check for when the quote is being escape by a many \
pub fn parse_record(l: &str) -> Option<record::Record> {
    let chars = l.as_bytes();
    let mut left_bound = next_non_space_char(chars, 0).unwrap();
    if chars[left_bound] != b'{' {
        return None;
    }
    left_bound += 1;

    let mut right_bound: usize;
    let mut label_pairs = Vec::new();
    loop {
        left_bound = next_non_space_char(chars, left_bound).unwrap();
        right_bound = find_next(chars, left_bound, b'=').unwrap();
        let key = &chars[left_bound..right_bound];
        left_bound = find_next(chars, left_bound, b'"').unwrap() + 1;

        // Search for next instance of " that is not escaped
        right_bound = find_next(chars, left_bound, b'"').unwrap();
        while is_escaped(chars, right_bound) {
            right_bound = find_next(chars, right_bound + 1, b'"').unwrap();
        }

        let val = &chars[left_bound..right_bound];
        left_bound = right_bound + 1;
        let lp = record::LabelPair {
            key: Box::from(str::from_utf8(key).unwrap().trim_end()),
            val: Box::from(str::from_utf8(val).unwrap()),
        };
        label_pairs.push(lp);

        left_bound = next_non_space_char(chars, left_bound).unwrap();

        match chars[left_bound] {
            b',' => left_bound += 1,
            b'}' => break,
            _ => return None,
        };
    }
    return Some(record::Record { label_pairs });
}

#[inline]
pub fn parse_small_record(l: &str) -> Option<record::SmallRecord> {
    let chars = l.as_bytes();
    let mut left_bound = next_non_space_char(chars, 0).unwrap();
    if chars[left_bound] != b'{' {
        return None;
    }
    left_bound += 1;

    let mut right_bound: usize;
    let mut label_pairs = SmallVec::new();
    loop {
        left_bound = next_non_space_char(chars, left_bound).unwrap();
        right_bound = find_next(chars, left_bound, b'=').unwrap();
        let key = &chars[left_bound..right_bound];
        left_bound = find_next(chars, left_bound, b'"').unwrap() + 1;

        // Search for next instance of " that is not escaped
        right_bound = find_next(chars, left_bound, b'"').unwrap();
        while is_escaped(chars, right_bound) {
            right_bound = find_next(chars, right_bound + 1, b'"').unwrap();
        }

        let val = &chars[left_bound..right_bound];
        left_bound = right_bound + 1;
        let lp = record::SmallLabelPair {
            key: SmallString::from_str(str::from_utf8(key).unwrap().trim_end()),
            val: SmallString::from_str(str::from_utf8(val).unwrap().trim_end()),
        };
        label_pairs.push(lp);

        left_bound = next_non_space_char(chars, left_bound).unwrap();

        match chars[left_bound] {
            b',' => left_bound += 1,
            b'}' => break,
            _ => return None,
        };
    }
    return Some(record::SmallRecord { label_pairs });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let field = "{author_family_name=\"Daniels\", author_first_name=\"B\",author_surname=\"J\", language=\"English\", year=\"0\", extension=\"rar\", title=\"Stolen Moments\",publisher=\"\",edition=\"\"}";

        assert!(parse_record(field).is_some())
    }

    #[test]
    fn it_works_with_quote() {
        let quote_field = "{author_family_name=\"Dan\\\"iels\"}";
        parse_record(quote_field);
        assert!(parse_record(quote_field).is_some());
        let escaped_quote_field = "{author_family_name=\"Dan\\\"iels\\\\\"}";
        parse_record(escaped_quote_field);
        assert!(parse_record(escaped_quote_field).is_some());
    }
}
