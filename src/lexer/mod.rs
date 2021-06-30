use crate::record::query;
use crate::record;
use smallstr::SmallString;
use smallvec::SmallVec;
use std::str;
use logos::{Lexer, Logos};

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


#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[token("{")]
    OpeningBraces,
    #[token("}")]
    ClosingBraces,
    #[token("(")]
    OpeningParenthesis,
    #[token(")")]
    ClosingParenthesis,
    #[token("=")]
    Equal,
    #[token("==")]
    DoubleEqual,
    #[token("=~")]
    TildeEqual,
    #[token(",")]
    Comma,

    #[token("label_values")]
    FnLabelValues,

    #[regex("[a-zA-Z-_]+")]
    Literal,

    // TL;DR: parse a string enclosed in quotes, works with escaped quotes as well
    #[regex(r#""(?:[^"\\]|\\.)*""#)]
    ValueLiteral,

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,
}


fn parse_labels(lex: &mut Lexer<Token>) -> Option<SmallVec<[record::SmallLabelPair; 16]>>  {
    let mut label_pairs = SmallVec::new();
    loop {  
        let key = match lex.next() {
            Some(Token::Literal) => lex.slice(),
            _ => break,
        };

        match lex.next() {
            Some(Token::Equal) => (),
            _ => break,
        };

        let val   = match lex.next() {
            Some(Token::ValueLiteral) => lex.slice().strip_prefix('"').unwrap().strip_suffix('"').unwrap(),
            _ => break,
        };
        let lp = record::SmallLabelPair {
            key: SmallString::from_str(key),
            val: SmallString::from_str(val),
        };
        label_pairs.push(lp);

        match lex.next() {
            Some(Token::Comma) => continue,
            Some(Token::ClosingBraces) => return Some(label_pairs),
            _ => break,
        };

    }
    return None
}


#[inline]
fn parse_search_fields(lex: &mut Lexer<Token>) -> Option<Vec<query::Field>> {
    let mut fields = Vec::new();
    loop {  
        let key = match lex.next() {
            Some(Token::Literal) => lex.slice(),
            _ => break,
        };

        let op = match lex.next() {
            Some(Token::DoubleEqual) => query::Operation::Eq,
            Some(Token::TildeEqual) => query::Operation::Re,
            _ => break,
        };

        let val   = match lex.next() {
            Some(Token::ValueLiteral) => lex.slice().strip_prefix('"').unwrap().strip_suffix('"').unwrap(),
            _ => break,
        };
        let lp = query::Field {
            key: Box::from(key),
            val: Box::from(val),
            op
        };
        fields.push(lp);

        match lex.next() {
            Some(Token::Comma) => continue,
            Some(Token::ClosingBraces) => return Some(fields),
            _ => break,
        };

    }
    return None
}

#[inline]
fn parse_fn_search_fields(lex: &mut Lexer<Token>) -> Option<query::Query> {
    let search_fields = match parse_search_fields(lex) {
        Some(x) => x,
        None => return None,
    };
    return Some(query::Query::Simple(query::Search { search_fields, query_flags: query::SearchFlags::DEFAULT }));
}

#[inline]
fn parse_fn_label_values(lex: &mut Lexer<Token>) -> Option<query::Query> {
    match lex.next() {
        Some(Token::OpeningParenthesis) => (),
        _ => return None,
    };
    match lex.next() {
        Some(Token::OpeningBraces) => (),
        _ => return None,
    };
    let search_fields = match parse_search_fields(lex) {
        Some(x) => x,
        None => return None,
    };
    match lex.next() {
        Some(Token::Comma) => (),
        _ => return None,
    };
    let key_field = match lex.next() {
        Some(Token::ValueLiteral) => lex.slice().strip_prefix('"').unwrap().strip_suffix('"').unwrap(),
        _ => return None,
    };
    match lex.next() {
        Some(Token::ClosingParenthesis) => (),
        _ => return None,
    };
    return Some(query::Query::KeyValues(query::KeyValuesSearch { search_fields, query_flags: query::SearchFlags::DEFAULT, key_field: Box::from(key_field) }));
}


#[inline]
pub fn parse_record(l: &str) -> Option<record::SmallRecord> {
    let mut lex = Token::lexer(l);
    let label_pairs = match lex.next() {
        Some(Token::OpeningBraces) => parse_labels(&mut lex),
        _ => return None,
        
    }.unwrap();
    return Some(record::SmallRecord { label_pairs });
}  

#[inline]
pub fn parse_query(l: &str) -> Option<query::Query> {
    let mut lex = Token::lexer(l);
    match lex.next() {
        Some(Token::OpeningBraces) => parse_fn_search_fields(&mut lex),
        Some(Token::FnLabelValues) => parse_fn_label_values(&mut lex),
        _ => None,  
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_record_works() {
        let field = r#"{author_family_name="Daniels", author_first_name="B",author_surname="J", language="English", year="0", extension="rar", title="Stolen Moments",publisher="",edition=""}"#;
        assert!(parse_record(field).is_some())
    }

    #[test]
    fn parse_record_works_with_quote() {
        let quote_field = "{author_family_name=\"Dan\\\"iels\"}";
        parse_record(quote_field);
        assert!(parse_record(quote_field).is_some());
        let escaped_quote_field = "{author_family_name=\"Dan\\\"iels\\\\\"}";
        parse_record(escaped_quote_field);
        assert!(parse_record(escaped_quote_field).is_some());
    }

    #[test]
    fn parse_query_works() {
        let mut field = parse_query(r#"{author_family_name=="Tolkien", language=~"English", extension=="epub"}"#);
        assert!(field.is_some());
        field = parse_query(r#"label_values({author_family_name=="Tolkien", language=~"English", extension=="epub"}, "extension")"#);
        assert!(field.is_some());
        match field.unwrap() {
            query::Query::KeyValues(x) => assert!(x.key_field == Box::from("extension")),
            _ => panic!("Wrong query parsed"),
        };
    }
}
