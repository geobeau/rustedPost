use super::record;
use hashbrown::HashMap;
use std::{collections::BTreeMap};
use iter_set;
use regex::Regex;

/// Index contains a map of field name to field
/// A field contains a map of 
pub struct Index {
    label_key_index: HashMap<Box<str>, Field>
}

impl Index {
    pub fn new() -> Index {
        Index {label_key_index: HashMap::new()}
    }

    pub fn search(&self, query: record::SearchQuery) -> Vec<usize> {
        // TODO: generate a result instead of empty vec

        // Key search phase
        // Get the list of possible values from the index for each keys
        let key_search: Option<Vec<_>> = query.search_fields.into_iter().map(|query| {
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
                record::Operation::Re => q.1.re_aggregated_get(&q.0),
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

    fn re_aggregated_get(&self, field_query: &record::SearchField) -> Option<Vec<usize>> {
        // TODO: generate a result instead of option
        let re = Regex::new(&field_query.val).unwrap();
        Some((&self.field_map).into_iter().fold( Vec::new(), |a, b| {
            if re.is_match(&b.0) {
                let res = iter_set::union(&a, b.1).map(|a| a.clone()).collect();
                return res
            }
            a
        }))
    }

    fn eq_get(&self, field_query: &record::SearchField) -> Option<Vec<usize>> {
        match self.field_map.get(&*field_query.val) {
            Some(list) => Some(list.clone()),
            None => None
        }
    }
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

        let mut result = index.search(record::SearchQuery{search_fields: vec![record::SearchField::new_eq("keya", "val1")]});
        assert_eq!(result, vec![0, 1, 2]);
        result = index.search(record::SearchQuery{search_fields: vec![record::SearchField::new_eq("keyb", "val1")]});
        assert_eq!(result, vec![0, 2]);
    }

    #[test]
    fn it_intersects() {
        let mut index = Index::new();
        load_test_data(&mut index);

        let mut result = index.search(record::SearchQuery{search_fields: vec![record::SearchField::new_eq("keya", "val1"), record::SearchField::new_eq("keya", "val1")]});
        assert_eq!(result, vec![0, 1, 2]);
        result = index.search(record::SearchQuery{search_fields: vec![record::SearchField::new_eq("keya", "val1"), record::SearchField::new_eq("keyb", "val1")]});
        assert_eq!(result, vec![0, 2]);
        result = index.search(record::SearchQuery{search_fields: vec![record::SearchField::new_eq("keyc", "val3"), record::SearchField::new_eq("keyb", "val1")]});
        assert_eq!(result, vec![0]);
    }
}