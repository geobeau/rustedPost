use std::{collections::HashMap, vec};

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

    fn search(&mut self, record: Record) {
        // let t = record.labelPair.into_iter().map(|pair| {
        //     (pair.val, self.labelKeyIndex.get(&pair.key))
        // })
        // .filter field not None // TODO
        // .map(|(val, field)| {
        //     field.field_map.get(val)
        // });
        println!("ok");
        //.reduce(intesection)
    }

    fn insert_record(&mut self, id: usize, record: &Record) {
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

// https://doc.rust-lang.org/std/default/trait.Default.html

fn main() {
    let mut store = RecordStore::new();
    let mut index = Index::new();
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
    let mut id = store.add(&record1);
    index.insert_record(id, &record1);

    id = store.add(&record2);
    index.insert_record(id, &record2);
    
    let record_search = Record{
        labelPair: vec![LabelPair{key: String::from("color"), val: String::from("green")}]
    };
    index.search(record_search)
}

fn generator() {

}
