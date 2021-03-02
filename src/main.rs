use std::collections::HashMap;

#[derive(Clone)]
struct Record {
    labelPair: Vec<labelPair>,
}

#[derive(Clone)]
struct labelPair {
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

    fn add(&mut self, record: Record) -> usize {
        self.store.push(record);
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
    index_map: HashMap<String, Field>
}

impl Index {
    fn new() -> Index {
        Index {index_map: HashMap::new()}
    }

    fn get(&mut self, index_name: String) -> Field {
        match (*self).index_map.get(&index_name) {
            Some(x) => x.clone(),
            None => Field::new(),
        }
    }

    fn insert_record(&mut self, id: usize, record: &Record) {
        for i in &record.labelPair {
            field = self.get(labelPair.key)
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
        (*self).index_map.insert(index_name, field);
    }
}
// https://doc.rust-lang.org/book/ch15-04-rc.html
// https://doc.rust-lang.org/std/sync/struct.Arc.html

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

    let mut field = Field::new();
    field.field_map.insert(
        String::from("banane"),
        [1; 32],
    );

    let mut index = Index::new();

    index.insert(
        String::from("fruit"),
        field,
    );
    let result = index.get(String::from("fruit"));
    match result.field_map.get(&String::from("banane")) {
        Some(x) => println!("{}", x[0]),
        None => println!("not find"),
    };
    // assert!(result.field_map == [1; 32], "ok");

    // let mut couleurIndex = HashMap::new();
    // couleurIndex.insert(
    //     "tomate",
    //     [0; 32],
    // );
    // couleurIndex.insert(
    //     "banane",
    //     [0; 32],
    // );
    // couleurIndex.insert(
    //     "kiwi",
    //     [0; 32],
    // );
    // index.index_map.insert(
    //     "fruit",
    //     fruitIndex,
    // );
    // index.index_map.insert(
    //     "couleur",
    //     couleurIndex,
    // );

    // let mut posting_list: [i32; 32] = [0; 32];
}
