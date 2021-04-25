use super::record;
use hashbrown::HashMap;
use iter_set::intersection;

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
        let key_iter: Option<Vec<_>> = query.search_fields.into_iter().map(|query| {
            match self.label_key_index.get(&query.key) {
                Some(field) => Some((query.val, field)),
                None => None
            }
        }).collect();

        if key_iter.is_none() {
            return Vec::new();
        }

        let mut t = key_iter.unwrap().into_iter().filter_map(|q| {
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
        for pair in &record.label_pairs {
            let field = self.label_key_index.entry(pair.key.clone()).or_insert(Field::new());
            field.add_posting(pair.val.clone(), id);
        }
    }
}

#[derive(Clone)]
struct Field {
    field_map: HashMap<Box<str>, Vec<usize>>
}

impl<'a> Field {
    fn new() -> Field {
        Field {field_map: HashMap::new()}
    }

    fn add_posting(&mut self, key: Box<str>, id: usize) {
        let posting_list = self.field_map.entry(key).or_insert(Vec::new());
        posting_list.push(id);
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