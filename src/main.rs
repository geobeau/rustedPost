use std::collections::HashMap;

#[derive(Clone)]
struct Record {
    labelPair: Vec<LabelPair>,
}

#[derive(Clone)]
struct LabelPair {
    key: String,
    val: String
}

struct RecordStore {
    store: Vec<Record>
}

impl RecordStore {
    fn new() -> RecordStore {
        RecordStore {store: Vec::new()}
    }

    fn add(&mut self, record: &Record) -> usize {
        self.store.push(record.clone());
        self.store.len() -1
    }

    fn get(&mut self, index: usize) -> Option<Record> {
        match self.store.get(index) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
}

/// Index contains a map of field name to field
/// A field contains a map of 
struct Index {
    labelKeyIndex: HashMap<String, Field>
}

impl Index {
    fn new() -> Index {
        Index {labelKeyIndex: HashMap::new()}
    }

    fn get(&mut self, index_name: String) -> Field {
        match (*self).labelKeyIndex.get(&index_name) {
            Some(x) => x.clone(),
            None => Field::new(),
        }
    }

    fn insert_record(&mut self, id: &usize, record: &Record) {
        for pair in &record.labelPair {
            let field = self.labelKeyIndex.get(&pair.key);
            // TODO get val if exist append id
            match field.get(&labelPair.val) {
                Some(x) => [*x, id], -> //TODO transform in Vec
                None => [id],
            }
            field.insert(
                labelPair.val,
                //id,
            )
            self.insert(labelPair.key, field);
        }
    }

    fn insert(&mut self, index_name: String, field: Field) {
        (*self).labelKeyIndex.insert(index_name, field);
    }
}

#[derive(Clone)]
struct Field {
    field_map: HashMap<String, [usize; 32]>
}

impl<'a> Field {
    fn new() -> Field {
        Field {field_map: HashMap::new()}
    }
}

// https://doc.rust-lang.org/std/default/trait.Default.html

fn main() {
    // Print text to the console
    println!("Hello World!");

    let store = RecordStore::new();
    let index = Index::new();
    let record1 = Record{
        labelPair: vec![LabelPair{key: String::from("type"), val: String::from("fruit")}, 
                        LabelPair{key: String::from("color"), val: String::from("green")},
                        LabelPair{key: String::from("name"), val: String::from("kiwi")}]
    };
    let record2 = Record{
        labelPair: vec![LabelPair{key: String::from("type"), val: String::from("vegetable")}, 
                        LabelPair{key: String::from("color"), val: String::from("green")},
                        LabelPair{key: String::from("name"), val: String::from("bean")}]
    };
    let id = store.add(&record1);
    index.insert_record(&id, &record1);

    id = store.add(&record2);
    index.insert_record(&id, &record2);
}

fn generator() {

}
