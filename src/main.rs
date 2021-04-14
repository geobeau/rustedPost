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
    let record3 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("type"), val: String::from("fruit")}, 
                        record::LabelPair{key: String::from("color"), val: String::from("red")},
                        record::LabelPair{key: String::from("name"), val: String::from("strawberry")}]
    };
    let mut id = store.add(&record1);
    index.insert_record(id, &record1);

    id = store.add(&record2);
    index.insert_record(id, &record2);
    id = store.add(&record3);
    index.insert_record(id, &record3);
    
    let record_search1 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("color"), val: String::from("green")}]
    };
    let record_search2 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("name"), val: String::from("strawberry")}]
    };
    let mut records: Vec<record::Record> = store.multi_get(index.search(record_search1));
    println!("Search 1: {:?}", records);
    records = store.multi_get(index.search(record_search2));
    println!("Search 2: {:?}", records);
}
