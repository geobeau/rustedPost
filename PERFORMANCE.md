#  Baseline

The code is always compiled with optimization at 3 and debug info.

Computer used:
* MacBook Pro 2017
* 2,8 GHz Quad-Core Intel Core i7
* 16GB memory

```
Loaded 1952967 out of 2206073 lines in 21_876ms
Search 1 (Tolkien): yielded 1318 results in 3us
Search 2 (Tolkien in English): yielded 724 results in 15_123us
Search 2 (Tolkien in English and as pdf): yielded 78 results in 6_229us
```

Memory usage is computed by running (output is in GiB or MiB):
```
ps x -o rss,vsz,command | grep -i target/debug/rusted_post | grep -v grep | numfmt --from-unit=1024 --to=iec --field 1-2
```

# Multithreading

The code is single threaded. The reason is that if clustering is added, one
possible implementation is to shard the data and have a single server host
multiple shard. Shards would then be pinned to a specific CPU, hopefully 
benefiting from data locality and being lock free.

# Memory usage

Going from `String` to `Box<str>` reduced memory usage by 25% (2.6GiB to 2.0GiB).

TODO:
In a record we store a key and a label. The key is dynamic but it may have low cardinality compared
to values (`author_name` should be much more common than `tolkien`). So instead of repeating
it many times we can declare the key as a RC and store them into an hashmap somewhere.

# Ingestion

Ingestion takes 1s per 100 000 records. 

## Serialization

Right now the serialisation is done with
JSON which is not the most efficient, doing it with a binary format would yield better result.
However, serialisation takes only ~20% of cpu usage and JSON has the benefit of being human
readable and writable so it's good enough for now.

## Datastructures & copy

First optimisation is to use https://github.com/rust-lang/hashbrown instead of the std hashmap.
The main benefit for ingestion seems to the faster hash function, it helps reducing ingestion time
by ~10% (from 18s to 16s) without downsides.

Second optimisation is avoid copying data. A record is stored as a RC record in the heap and we
just move the reference around.

# Queries

## Regex search

Searching with a regex pattern on the value is very useful but very costly if done on field with
high cardinality. The reason is that we have to scan all the values to find the ones matching.

Example:
```
Searching (author_family_name=="Tolkien", language=="English", title=~"[sS]ilmarillion"): yielded 44 results in 103920us (103ms)
Searched with [sS]ilmarillion over 954990 values, matched 21 (ratio 0.00002198975905506864)
```

In this case we searched around 1M line just to match 21 records.

It's possible to be much efficient by analysing the regex and get:
- Either a list of exact matches
- A common suffix

the crate regex-syntax used by the main Rust regex library provide some useful function. A regex can be parsed to its AST
(Abstract Syntax Tree) then to the its high-level intermediate representation (HIR). The HIR is in between the compiled
regex and the AST. It's useful because it's a cleaned and already processed version of the regex.

The HIR as method to extract a list of literrals which can be marked as Cut (prefix) or Complete (exact match).

We then can do a sub range query over the values using the prefixes or match direcly using the exact matches.

### Prefix optimization

In this example, we are searching for the book Anna Karenina from Tolstoy. To account for different writing styles (Anna is ana
in spanish and Karenina is spelled Karénine in French), we use the following regex: `"A[n]?na.*`.


```
[2021-04-28T21:27:21][DEBUG][src\storage\index.rs:123] Searched with A[n]?na.* over 954990 values, matched 706 (ratio 0.0007392747568037362)
[2021-04-28T21:27:21][INFO][src\main.rs:17] Searching (author_family_name=="Tolstoy", title=~"A[n]?na.*"): yielded 40 results in 94065us (94ms) (optimized: false)
```

The naive (non optimized version) is quite slow (94ms). We had to loop over 1M book titles trying to match each with our regex.

```
[2021-04-28T21:27:21][DEBUG][src\storage\index.rs:83] Running query in optimized mod
[2021-04-28T21:27:21][DEBUG][src\storage\index.rs:86] Search for Anna (cut:true)
[2021-04-28T21:27:21][DEBUG][src\storage\index.rs:86] Search for Ana (cut:true)
[2021-04-28T21:27:21][DEBUG][src\storage\index.rs:123] Searched with A[n]?na.* over 706 values, matched 706 (ratio 1)
[2021-04-28T21:27:21][INFO][src\main.rs:17] Searching (author_family_name=="Tolstoy", title=~"A[n]?na.*"): yielded 40 results in 2116us (2ms) (optimized: true)
```

The optimized version is much faster (2ms). The magic is done because the optimizer is able to see that we can extract
two prefix (cut mean that it's not an exact match). With the two prefix we can do a range search on the tittles (they
are in a BTreeMap instead of a Hashmap for this purpose). In our case we interated over 706 titles and they all matched.

### Exact match optimization

For this example we will only search for Tolkien but account for lowercase as well: `[tT]olkien`


```
[2021-04-28T21:39:59][DEBUG][src\storage\index.rs:123] Searched with [tT]olkien over 99779 values, matched 2 (ratio 0.000020044297898355367)
[2021-04-28T21:39:59][INFO][src\main.rs:17] Searching (author_family_name=~"[tT]olkien"): yielded 1319 results in 9376us (9ms) (optimized: false)
```

The naive (non optimized version) is rather fast (10ms) but we still had to loop over 100k author names trying
to find something matching the regex.

```
[2021-04-28T21:39:59][DEBUG][src\storage\index.rs:83] Running query in optimized mod
[2021-04-28T21:39:59][DEBUG][src\storage\index.rs:86] Search for Tolkien (cut:false)
[2021-04-28T21:39:59][DEBUG][src\storage\index.rs:86] Search for tolkien (cut:false)
[2021-04-28T21:39:59][DEBUG][src\storage\index.rs:123] Searched with [tT]olkien over 2 values, matched 2 (ratio 1)
[2021-04-28T21:39:59][INFO][src\main.rs:17] Searching (author_family_name=~"[tT]olkien"): yielded 1319 results in 718us (0ms) (optimized: true)
```

The optimized version is even faster (0.7ms). The optimizer saw that the regex could be described
as two exacts matches, bypassing the need for a range search at all.


