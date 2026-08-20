# Sthirayama Benchmarks

Hardware:
- CPU: AMD Ryzen 5 7533HS with Radeon Graphics
- RAM: 12 GiB
- OS: Fedora Linux
- Rust: rustc 1.94.0

## Baseline — before Bloom Filter

Commit: `2528648`

Command:

```bash
cargo bench --package engine --bench engine_bench
```

| Benchmark | Time |
| :--- | :--- |
| **set** | 16.0 µs |
| **get_memtable** | 156 ns |
| **get_sstable** | 505 µs |
| **get_missing** | 130 ns |
| **set_with_flush** | 15.6 ms |

Notes:

- MemTable lookup is ~156 ns.
- SSTable lookup is ~505 µs.
- Missing-key benchmark currently tests the existing missing-key path.
