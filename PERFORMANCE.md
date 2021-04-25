#  Baseline

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

# Multithreading

The code is single threaded. The reason is that if clustering is added, one
possible implementation is to shard the data and have a single server host
multiple shard. Shards would then be pinned to a specific CPU, hopefully 
benefiting from data locality and being lock free.

# Ingestion

Ingestion takes 1s per 100 000 records. Right now the serialisation is done with
JSON which is not the most efficient, doing it with a binary format would yield better result.
However, serialisation takes only ~20% of cpu usage and JSON has the benefit of being human
readable and writable so it's good enough for now.




