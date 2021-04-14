use std::vec;

mod record;
mod store;
mod index;

// https://doc.rust-lang.org/std/default/trait.Default.html

fn main() {
    let mut store = store::RecordStore::new();
    let mut index = index::Index::new();
    let record1 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("type"), val: String::from("fruit")}, 
                        record::LabelPair{key: String::from("color"), val: String::from("green")},
                        record::LabelPair{key: String::from("name"), val: String::from("kiwi")}]
    };
    let record2 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("type"), val: String::from("vegetable")}, 
                        record::LabelPair{key: String::from("color"), val: String::from("green")},
                        record::LabelPair{key: String::from("name"), val: String::from("bean")}]
    };
    let mut id = store.add(&record1);
    index.insert_record(id, &record1);

    id = store.add(&record2);
    index.insert_record(id, &record2);
    
    let record_search = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("color"), val: String::from("green")}]
    };
    index.search(record_search);
}
