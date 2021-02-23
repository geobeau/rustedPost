use std::collections::HashMap;

// This is the main function
fn main() {
    // Statements here are executed when the compiled binary is called

    // Print text to the console
    println!("Hello World!");
    let mut index = HashMap::new();
    index.insert(
        "Pride and Prejudice",
        "Very enjoyable.",
    );
    index.entry("fruit").or_insert("poivron");

    let mut posting_list: [i32; 32] = [0; 32];
}
