use super::record;
use std::{collections::HashMap};
use iter_set::intersection;

/// Index contains a map of field name to field
/// A field contains a map of 
pub struct Index {
    label_key_index: HashMap<String, Field>
}

impl Index {
    pub fn new() -> Index {
        Index {label_key_index: HashMap::new()}
    }

    pub fn search(&self, record: record::Record) -> Vec<usize> {
        let mut t = record.label_pair.into_iter().filter_map(|pair| {
            match self.label_key_index.get(&pair.key) {
                Some(field) => Some((pair.val, field)),
                None => None
            }
        }).filter_map(|q| {
            let field = q.1;
            field.field_map.get(&q.0)
        });
        let last = t.next_back();
        if last.is_none() {
            return Vec::new();
        }
        t.fold(last.unwrap().clone(), |a, b| {
            intersection(&a, b).map(|a| a.clone()).collect()
        })
    }

    pub fn insert_record(&mut self, id: usize, record: &record::Record) {
        for pair in &record.label_pair {
            let field = self.label_key_index.entry(pair.key.clone()).or_insert(Field::new());
            field.add_posting(pair.val.clone(), id);
        }
    }
}

#[derive(Clone)]
struct Field {
    field_map: HashMap<String, Vec<usize>>
}

impl<'a> Field {
    fn new() -> Field {
        Field {field_map: HashMap::new()}
    }

    fn add_posting(&mut self, key: String, id: usize) {
        let posting_list = self.field_map.entry(key).or_insert(Vec::new());
        posting_list.push(id);
    }
}
