# Rusted Post

Rusted post is an index/search engine. It's goal is to quickly find record that matched two labels.

# How to use

As of today, it's not usable as is as the data ingestion and search are hardcoded.

```
cargo run .
```

# Generating the dataset

Checkout the code in `data/generate_from_lib_gen.py`, it requires a dump of the lib genesis index
that is fed to an sqlite database (the dump is a mysql compatible dump, so some conversion is required
clean output).
The dataset is a list of 2M books with authors, langage and format information. This dataset is then
used as benchmark reference to search things like "all the books by Tolkien in pdf and in english"

# Performance

As the goal of this project is to optimize the code for search performance and memory usage, you can
check the optimisation that were done in `PERFORMANCE.md`