use std::collections::HashMap;

// #![feature(entry_or_default)]
// use std::collections::HashMap;

// let mut map: HashMap<&str, Option<u32>> = HashMap::new();
// map.entry("poneyland").or_default();

//assert_eq!(map["poneyland"], None);

struct Index {
    index_map: HashMap<String, Field>
}

impl Index {
    fn new() -> Index {
        Index {index_map: HashMap::new()}
    }

    fn get(&mut self, index_name: String) -> Field {
        match (*self).index_map.get(&index_name) {
            Some(x) => *x,
            None => Field::new(0),
        }
    }

    fn insert(&mut self, index_name: String, field: Field) {
        (*self).index_map.insert(index_name, field);
    }
}

// https://doc.rust-lang.org/book/ch15-04-rc.html
// https://doc.rust-lang.org/std/sync/struct.Arc.html

#[derive(Default, Copy, Clone)]
struct Field {
    field_map: [usize; 32] //HashMap<String, [usize; 32]>
}

impl<'a> Field {
    fn new(val: usize) -> Field {
        Field {field_map: [val;32]}
    }
}

// https://doc.rust-lang.org/std/default/trait.Default.html

fn main() {
    // Statements here are executed when the compiled binary is called

    // Print text to the console
    println!("Hello World!");

    let mut index = Index::new();

    index.insert(
        String::from("fruit"),
        Field::new(1),
    );
    let result = index.get(String::from("fruit"));
    println!("{}", result.field_map[0]);
    assert!(result.field_map == [1; 32], "ok");

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
