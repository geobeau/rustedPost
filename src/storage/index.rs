use super::record;
use hashbrown::HashMap;
use std::vec;
use std::{collections::BTreeMap};
use regex::Regex;
use regex_syntax::Parser;
use regex_syntax::hir::literal::{Literals};
use log::{debug};
use roaring::RoaringBitmap;


pub enum KeyValuesSearchResult {
    Err(&'static str),
    Ok(Vec<Box<str>>),
    DirtyOk(Vec<u32>),
}

/// Index contains a map of field name to field
/// A field contains a map of 
pub struct Index {
    label_key_index: HashMap<Box<str>, Field>
}

impl Index {
    pub fn new() -> Index {
        Index {label_key_index: HashMap::new()}
    }

    pub fn key_values_search(&self, query: &record::KeyValuesSearch) -> KeyValuesSearchResult {
        let records = self.simple_search(&query.to_search_query());
    
        let field = self.label_key_index.get(&query.key_field);
        if field.is_none() {
            return KeyValuesSearchResult::Ok(Vec::new());
        }

        let map = &field.unwrap().field_map;
        if query.query_flags.contains(record::QueryFlags::ABORT_EARLY) && map.len() as u64 > records.len() {
            return KeyValuesSearchResult::DirtyOk(records.iter().collect());
        }

        KeyValuesSearchResult::Ok(map.iter().filter_map(|field| {
            let res = &records & field.1;
            if res.is_empty() {
                None
            } else {
                Some(field.0.clone())
            }
        }).collect())
    }

    fn simple_search(&self, query: &record::SearchQuery) -> RoaringBitmap {
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
            return RoaringBitmap::new();
        }

        let mut t = key_search.unwrap().into_iter().map(|q| {
            match q.0.op {
                record::Operation::Re => q.1.re_aggregated_get(&q.0, &query.query_flags),
                record::Operation::Eq => q.1.eq_get(&q.0)
            }
        });

        let last = t.next_back();
        if last.is_none() {
            return RoaringBitmap::new();
        }
        t.fold(last.unwrap(), |a, b| {
            // TODO: Break early if the bitmap a is empty
            a & b
        })
    }

    pub fn search(&self, query: &record::SearchQuery) -> Vec<u32> {
        self.simple_search(query).iter().collect()
    }

    pub fn insert_record(&mut self, id: u32, record: &record::Record) {
        for pair in &record.label_pairs {
            let field = self.label_key_index.entry(pair.key.clone()).or_insert(Field::new());
            field.add_posting(pair.val.clone(), id);
        }
    }
}

#[derive(Clone)]
struct Field {
    field_map: BTreeMap<Box<str>, RoaringBitmap>
}

impl<'a> Field {
    fn new() -> Field {
        Field {field_map: BTreeMap::new()}
    }

    fn add_posting(&mut self, key: Box<str>, id: u32) {
        let posting_list = self.field_map.entry(key).or_insert(RoaringBitmap::new());
        posting_list.insert(id);
    }

    fn re_aggregated_get(&self, field_query: &record::SearchField, flags: &record::QueryFlags) -> RoaringBitmap {
        // TODO: generate a result instead of option
        let re = Regex::new(format!("^{}$", &field_query.val).as_str()).unwrap();
        let mut count = 0;
        let mut matched = 0;
        let mut result = RoaringBitmap::new();
        let optimized_fields = optimize_regex(&field_query.val);
        if flags.contains(record::QueryFlags::OPTIMIZE_REGEX_SEARCH) && !optimized_fields.is_empty() {
            debug!("Running query in optimized mod");
            optimized_fields.into_iter()
            .for_each(|lit| {
                debug!("Search for {} (cut:{})", lit.1, lit.0);
                if lit.0 {
                    // If it's a prefix do a range search and fold along the way
                    (&self.field_map).range(lit.1.clone()..)
                    .take_while(|(k, _)| (**k).starts_with(&*lit.1.clone()))
                    .for_each(|field| {
                        count += 1;
                        if re.is_match(&field.0) {
                            result |= field.1;
                            matched += 1;
                        }
                    });
                } else {
                    count += 1;
                    matched += 1;
                    // If it's an exact match do a simple get
                    match self.field_map.get(&*lit.1) {
                        Some(list) => result |= list,
                        None => ()
                    }
                    
                }  
            });
        } else {
            (&self.field_map).into_iter().for_each(|b| {
                count += 1;
                if re.is_match(&b.0) {
                    result |= b.1;
                    matched += 1;
                }
            });
        }

        debug!("Searched with {} over {} values, matched {} (ratio {})", field_query.val, count, matched, matched as f64 / count as f64);
        result
    }

    fn eq_get(&self, field_query: &record::SearchField) -> RoaringBitmap {
        match self.field_map.get(&*field_query.val) {
            Some(list) => list.clone(),
            None => RoaringBitmap::new()
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
        (l.is_cut(), Box::from(re.captures(format!("{:?}", l).as_str()).unwrap().get(1).unwrap().as_str()))
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
        optimize_regex("aba");
        optimize_regex("a{3,9}");
        optimize_regex("^[sS]il.*");
        optimize_regex("marillion.*^");
        optimize_regex("(t|T)olkien");
        optimize_regex("[tT]olkien");
        optimize_regex(".*");
        optimize_regex("(tolkien+|tolkien)");
    }
}



