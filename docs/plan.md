A Distributed, LSM-Tree Based Key-Value Store
Instead of using Redis, build a highly optimized storage engine from scratch. This proves you understand disk I/O, memory mapping, and distributed consistency.

* The Architecture (Rust): Build an LSM-Tree (Log-Structured Merge-tree) storage engine. Implement MemTables (in-memory write buffers using a SkipList), Write-Ahead Logs (WAL) for crash recovery, and SSTables (Sorted String Tables) on disk.
* The Advanced Twist: Implement an asynchronous compaction strategy using tokio and Rayon to merge SSTables and purge deleted data without blocking incoming writes.
* The Go Network Layer: Build a distributed cluster coordinator in Go using a custom Raft consensus implementation to replicate data across nodes. Use gRPC and Protobuf for node-to-node communication.