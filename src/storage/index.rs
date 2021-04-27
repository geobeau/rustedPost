use super::record;
use hashbrown::HashMap;
use std::{collections::BTreeMap};
use iter_set;
use regex::Regex;
use regex_syntax::Parser;
use regex_syntax::hir::literal::{Literals, Literal};
use log::{debug, error, info, trace, warn};

/// Index contains a map of field name to field
/// A field contains a map of 
pub struct Index {
    label_key_index: HashMap<Box<str>, Field>
}

impl Index {
    pub fn new() -> Index {
        Index {label_key_index: HashMap::new()}
    }

    pub fn search(&self, query: &record::SearchQuery) -> Vec<usize> {
        // TODO: generate a result instead of empty vec

        // Key search phase
        // Get the list of possible values from the index for each keys
        let key_search: Option<Vec<_>> = query.search_fields.clone().into_iter().map(|query| {
            match self.label_key_index.get(&query.key) {
                Some(field) => Some((query, field)),
                None => None
            }
        }).collect();

        if key_search.is_none() {
            return Vec::new();
        }

        let mut t = key_search.unwrap().into_iter().filter_map(|q| {
            match q.0.op {
                record::Operation::Re => q.1.re_aggregated_get(&q.0, &query.query_flags),
                record::Operation::Eq => q.1.eq_get(&q.0)
            }
        });
        let last = t.next_back();
        if last.is_none() {
            return Vec::new();
        }
        t.fold((*last.unwrap()).to_vec().clone(), |a, b| {
            iter_set::intersection(&a, &b).map(|a| a.clone()).collect()
        })
    }

    pub fn insert_record(&mut self, id: usize, record: &record::Record) {
        for pair in &record.label_pairs {
            let field = self.label_key_index.entry(pair.key.clone()).or_insert(Field::new());
            field.add_posting(pair.val.clone(), id);
        }
    }
}

#[derive(Clone)]
struct Field {
    field_map: BTreeMap<Box<str>, Vec<usize>>
}

impl<'a> Field {
    fn new() -> Field {
        Field {field_map: BTreeMap::new()}
    }

    fn add_posting(&mut self, key: Box<str>, id: usize) {
        let posting_list = self.field_map.entry(key).or_insert(Vec::new());
        posting_list.push(id);
    }

    fn re_aggregated_get(&self, field_query: &record::SearchField, flags: &record::QueryFlags) -> Option<Vec<usize>> {
        // TODO: generate a result instead of option
        let re = Regex::new(&field_query.val).unwrap();
        let mut count = 0;
        let mut matched = 0;
        let result: Vec<usize>;
        if flags.contains(record::QueryFlags::OPTIMIZE_REGEX_SEARCH) {
            let lit = optimize_regex(&field_query.val);
            result = lit.into_iter().fold( Vec::new(), |a, b| {
                // If the literal is a prefix
                if b.0 {
                    let fields = (&self.field_map).range(b.1..)
                    .take_while(|(k, _)| k.into_string().starts_with(&b.1.into_string()))
                    .fold(a, |a, b| {
                        if re.is_match(&b.0) {
                            let res = iter_set::union(&a, b.1).map(|a| a.clone()).collect();
                            matched += 1;
                            return res
                        }
                        a
                    });
                    fields
                } else {
                    a
                }
            });
        } else {
            result = (&self.field_map).into_iter().fold( Vec::new(), |a, b| {
                count += 1;
                if re.is_match(&b.0) {
                    let res = iter_set::union(&a, b.1).map(|a| a.clone()).collect();
                    matched += 1;
                    return res
                }
                a
            });
        }

        debug!("Searched with {} over {} values, matched {} (ratio {})", field_query.val, count, matched, matched as f64 / count as f64);
        Some(result)
    }

    fn eq_get(&self, field_query: &record::SearchField) -> Option<Vec<usize>> {
        match self.field_map.get(&*field_query.val) {
            Some(list) => Some(list.clone()),
            None => None
        }
    }
}

/// A helper function to return the prefixes usable in the aggregated get
pub fn optimize_regex(regex: &str) -> Vec<(bool, Box<str>)> {
    // TODO: Move regex to lazy static
    let re_cut = Regex::new(r"^Cut\((.*)\)$").unwrap();
    let re_complete = Regex::new(r"^Complete\((.*)\)$").unwrap();

    let hir = Parser::new().parse(regex).unwrap();
    Literals::prefixes(&hir).literals().into_iter().map(|l| {
        // I didn't find a better way :'(, everything is private
        let re = if l.is_cut() {&re_cut} else {&re_complete};
        (l.is_cut(), Box::from(re.captures(format!("{:?}", l).as_str()).unwrap().get(0).unwrap().as_str()))
    }).collect()

}

#[cfg(test)]
mod tests {
    use super::*;
    fn load_test_data(index: &mut Index) {
        index.insert_record(0, &record::Record{label_pairs: vec![record::LabelPair::new("keya", "val1"), record::LabelPair::new("keyb", "val1"), record::LabelPair::new("keyc", "val3")]});
        index.insert_record(1, &record::Record{label_pairs: vec![record::LabelPair::new("keya", "val1"), record::LabelPair::new("keyb", "val2"), record::LabelPair::new("keyc", "val2")]});
        index.insert_record(2, &record::Record{label_pairs: vec![record::LabelPair::new("keya", "val1"), record::LabelPair::new("keyb", "val1"), record::LabelPair::new("keyc", "val1")]});
    } 

    #[test]
    fn it_works() {
        let mut index = Index::new();
        load_test_data(&mut index);

        let mut result = index.search(&record::SearchQuery::new(vec![record::SearchField::new_eq("keya", "val1")]));
        assert_eq!(result, vec![0, 1, 2]);
        result = index.search(&record::SearchQuery::new(vec![record::SearchField::new_eq("keyb", "val1")]));
        assert_eq!(result, vec![0, 2]);
    }

    #[test]
    fn it_intersects() {
        let mut index = Index::new();
        load_test_data(&mut index);

        let mut result = index.search(&record::SearchQuery::new(vec![record::SearchField::new_eq("keya", "val1"), record::SearchField::new_eq("keya", "val1")]));
        assert_eq!(result, vec![0, 1, 2]);
        result = index.search(&record::SearchQuery::new(vec![record::SearchField::new_eq("keya", "val1"), record::SearchField::new_eq("keyb", "val1")]));
        assert_eq!(result, vec![0, 2]);
        result = index.search(&record::SearchQuery::new(vec![record::SearchField::new_eq("keyc", "val3"), record::SearchField::new_eq("keyb", "val1")]));
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn it_optimizes_regex() {
        // TODO make that a real test
        optimize("aba");
        optimize("a{3,9}");
        optimize("^[sS]il.*");
        optimize("marillion.*^");
        optimize("(t|T)olkien");
        optimize("[tT]olkien");
        optimize(".*");
        optimize("(tolkien+|tolkien)");
    }
}



