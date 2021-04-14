use std::vec;

mod record;
mod storage;

// https://doc.rust-lang.org/std/default/trait.Default.html

fn main() {
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

    let mut storage = storage::StorageBackend::new();
    storage.add(&record1);
    storage.add(&record2);
    storage.add(&record3);
    
    let record_search1 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("color"), val: String::from("green")}]
    };
    let record_search2 = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("name"), val: String::from("strawberry")}]
    };
    let mut records = storage.search(record_search1);
    println!("Search 1: {:?}", records);
    records = storage.search(record_search2);
    println!("Search 2: {:?}", records);
}
