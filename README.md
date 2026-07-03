# sthirayama
Distributed LSM-Tree storage engine in Rust with a Go-based Raft consensus layer — async compaction via Tokio/Rayon, gRPC/Protobuf replication.

<H1>Skiplist memtable<H1>

<img src="./design/skiplist-expected.excalidraw.svg" width="300">
<img src="./design/skiplist-output.png" width="300">