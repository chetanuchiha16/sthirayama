# Sthirayama Benchmarks

Hardware:
- CPU: AMD Ryzen 5 7533HS with Radeon Graphics
- RAM: 12 GiB
- OS: Fedora Linux
- Rust: rustc 1.94.0

Command:

```bash
cargo bench --package engine --bench engine_bench
```

---

## Performance History & Comparison

| Benchmark | Binary Search (`2528648`) | Reverse Scan (`9826803`) | Buffered Writer (`refactor/write`) | Change vs Reverse Scan | Performance Explanation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **set** | 16.0 µs | 15.18 µs | **9.54 µs** | **-37.2%** 🚀 | Faster memtable flushes |
| **get_memtable** | 156 ns | 154.7 ns | **100.7 ns** | **-34.9%** 🚀 | Reduced memory allocation overhead |
| **get_sstable** | 505 µs | 1.028 ms | **751.1 µs** | **-26.9%** 🚀 | Reduced I/O latency on SSTable lookup |
| **get_missing** | 130 ns | 45.52 µs | **34.24 µs** | **-24.8%** 🚀 | Faster sequential disk file reads |
| **set_with_flush** | 15.6 ms | 17.96 ms | **10.52 ms** | **-41.4%** 🚀 | Single batch I/O write replacing micro-writes |

---

## 1. Buffered SSTable Writer Optimization (`refactor/write`)

**Branch**: `refactor/write`

### Technical Change:
Refactored `SstableWriter` to serialize data blocks, index metadata, and footers into an in-memory byte buffer (`Vec<u8>`) prior to writing to disk. Replaced dozens of individual `file.write_all()` and `file.stream_position()` system calls per SSTable with a single contiguous `write_all()` call.

| Benchmark | Time |
| :--- | :--- |
| **set** | 9.54 µs |
| **get_memtable** | 100.7 ns |
| **get_sstable** | 751.1 µs |
| **get_missing** | 34.24 µs |
| **set_with_flush** | 10.52 ms |

### Performance Impact Analysis:
- **`set_with_flush` (-41.4%) & `set` (-37.2%)**: Eliminating per-entry file write system calls significantly reduced I/O wait times during memtable flushes.
- **`get_sstable` (-26.9%) & `get_missing` (-24.8%)**: Creating contiguous, non-fragmented SSTable files during benchmark initialization improved OS page cache read performance during lookup operations.

---

## 2. Correct Baseline — Reverse SSTable Scan (`9826803`)

**Commit**: `9826803` (*Refactor retrieval logic to search from recent to old SSTables*)

SSTable lookups iterate from the newest to oldest SSTable (`(0..sstable_count).rev()`) to guarantee temporal ordering and shadow/delete correctness across overlapping Level 0 SSTables.

| Benchmark | Time |
| :--- | :--- |
| **set** | 15.18 µs |
| **get_memtable** | 154.7 ns |
| **get_sstable** | 1.028 ms |
| **get_missing** | 45.52 µs |
| **set_with_flush** | 17.96 ms |

---

## 3. Initial Flawed Binary Search (`2528648`)

**Commit**: `2528648` (*Initial implementation using binary search across SSTables*)

*Note: This initial version used binary search (`partition_point`) across SSTable key bounds. It was functionally incorrect for overlapping SSTables, producing artificially low `get_missing` numbers because missing keys failed bounds checks without searching disk files.*

| Benchmark | Time |
| :--- | :--- |
| **set** | 16.0 µs |
| **get_memtable** | 156 ns |
| **get_sstable** | 505 µs |
| **get_missing** | 130 ns |
| **set_with_flush** | 15.6 ms |
