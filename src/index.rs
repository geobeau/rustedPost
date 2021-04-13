use super::record;
use std::{collections::HashMap};

/// Index contains a map of field name to field
/// A field contains a map of 
pub struct Index {
    labelKeyIndex: HashMap<String, Field>
}

impl Index {
    pub fn new() -> Index {
        Index {labelKeyIndex: HashMap::new()}
    }

    pub fn search(&mut self, record: record::Record) {
        let t = record.labelPair.into_iter().map(|pair| {
            (pair.val, self.labelKeyIndex.get(&pair.key))
        });
        t.filter(|val| {
            !val.1.is_none()
        });
        // .filter field not None // TODO
        // .map(|(val, field)| {
        //     field.field_map.get(val)
        // });
        println!("ok");
        //.reduce(intesection)
    }

    pub fn insert_record(&mut self, id: usize, record: &record::Record) {
        for pair in &record.labelPair {
            let field = self.labelKeyIndex.entry(pair.key.clone()).or_insert(Field::new());
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
