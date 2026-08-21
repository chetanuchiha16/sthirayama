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

## Benchmark History & Comparison

| Benchmark | Flawed Baseline (`2528648`) | Correct Baseline (`9826803`) | Change | Impact Summary |
| :--- | :--- | :--- | :--- | :--- |
| **set** | 16.0 µs | 15.18 µs | -5.1% | Similar performance |
| **get_memtable** | 156 ns | 154.7 ns | -0.8% | In-memory skiplist lookup unaffected |
| **get_sstable** | 505 µs | 1.028 ms | +103.5% | Scans recent SSTables before hitting target |
| **get_missing** | 130 ns | 45.52 µs | +34,915% | Correctly scans **all** SSTables on disk |
| **set_with_flush** | 15.6 ms | 17.96 ms | +15.1% | Overhead of flushing and reverse-lookup validation |

---

## 1. Correct Baseline — Reverse Scan (`9826803`)

**Commit**: `9826803` (*Refactor retrieval logic to search from recent to old SSTables*)

This commit establishes the **correct Level 0 baseline**. SSTable lookups iterate from the newest to oldest SSTable (`(0..sstable_count).rev()`) to guarantee temporal ordering and shadow/delete correctness for overlapping SSTables.

| Benchmark | Time |
| :--- | :--- |
| **set** | 15.18 µs |
| **get_memtable** | 154.7 ns |
| **get_sstable** | 1.028 ms |
| **get_missing** | 45.52 µs |
| **set_with_flush** | 17.96 ms |

### Key Observations:
- **`get_missing` (45.52 µs)**: Serves as the primary target metric for **Bloom Filters**. Without filters, searching for a non-existent key requires opening and checking all SSTable files on disk.
- **`get_sstable` (1.028 ms)**: Serves as the target metric for **In-Memory Range Bounds (`min_key`/`max_key`)**. Skip files whose key range does not contain the searched key.

---

## 2. Initial Flawed Baseline (`2528648`)

**Commit**: `2528648` (*Initial baseline with global SSTable binary search*)

*Note: This version used a binary search (`partition_point`) across SSTable metadata. It was functionally broken for overlapping SSTables, resulting in artificially low `get_missing` times because missing keys failed bounds checks early without searching disk files.*

| Benchmark | Time |
| :--- | :--- |
| **set** | 16.0 µs |
| **get_memtable** | 156 ns |
| **get_sstable** | 505 µs |
| **get_missing** | 130 ns |
| **set_with_flush** | 15.6 ms |
