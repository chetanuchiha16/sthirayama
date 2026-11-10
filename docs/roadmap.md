# Sthirayama Roadmap

> **Sthirayama** (Sanskrit: *Sthira* / Stable · Japanese: *Yama* / Mountain)
> A high-performance, distributed, LSM-tree key-value store built in Rust and Go.

This document outlines the architectural roadmap, deployment modes, packaging & distribution channels, client SDKs, and engine performance optimizations.

---

## 1. The Four Deployment & Access Modes

Sthirayama is designed to decouple compute and storage from client tooling, supporting four primary consumption models:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        MODE 4: AGNOSTIC TUI / CLI                      │
│                  `sthirayama` (Bubble Tea Client)                      │
│      Connects via HTTP / gRPC to any backend mode without changes       │
└──────────────┬─────────────────────────┬───────────────────────────────┘
               │                         │
               ▼                         ▼
┌──────────────────────────────┐ ┌──────────────────────────────────────┐
│  MODE 1: SINGLE-NODE DAEMON  │ │     MODE 2: MULTI-NODE DOCKER        │
│  • Direct host binary        │ │  • `docker compose up -d`            │
│  • Systemd service           │ │  • 3-node Raft on single laptop      │
│  • Like redis-server/postgres│ │  • Isolated bridge network & volumes │
└──────────────────────────────┘ └──────────────────────────────────────┘
               │
               ▼
┌───────────────────────────────────────────────────────────────────────┐
│                     MODE 3: KUBERNETES CLUSTER                        │
│  • Multi-PC home lab (k3s across 2 machines) or Cloud (AWS/GCP)       │
│  • StatefulSet (3 replicas) + Headless Service + PVC disks            │
│  • Automatic self-healing, failover, and anti-affinity scheduling     │
└───────────────────────────────────────────────────────────────────────┘
```

### Mode 1: Single-Node Host Daemon (Direct Download)
* Zero Docker required. For users who want a lightweight, fast, local KV store (similar to `redis-server` or `sqlite`).
* Binary package installed directly to `/usr/local/bin/sthirayama-server`.
* Managed via `systemd` (`systemctl start sthirayama`) or ad-hoc CLI flag (`sthirayama-server --daemon`).

### Mode 2: Multi-Node Docker (Local Cluster)
* Single `docker-compose.yml` spinning up a complete 3-node Raft consensus cluster on a single laptop.
* Nodes communicate over an internal Docker network bridge (`node1:7001`, `node2:7001`, `node3:7001`).
* Host ports mapped (`8001`, `8002`, `8003`) for local testing, failover drills, and TUI connection.

### Mode 3: Kubernetes Distributed Deployment
* Cloud or multi-PC deployment (e.g., k3s running across 2 physical PCs).
* **StatefulSet** (`replicas: 3`) with stable network identities (`sthirayama-0`, `sthirayama-1`, `sthirayama-2`).
* Cloud Persistent Volumes (PVC) dynamically attached to each node.
* Resilient to hardware failures: if a machine goes offline, Kubernetes migrates the pod and reattaches storage while Raft re-elects a leader.

### Mode 4: Agnostic TUI / CLI
* Interactive terminal client (`sthirayama`) powered by Bubble Tea and the dynamic mascot.
* Completely backend-agnostic: does not care whether it connects to `localhost:8001` (Mode 1 or 2) or `k8s.cluster.local:8001` (Mode 3).
* Can be installed independently on client machines without installing the server backend.

---

## 2. Packaging & Distribution Channels

Make Sthirayama easily installable across all environments without needing Rust or Go toolchains:

### Linux Packages
- [ ] **RPM / DNF (Fedora, RHEL, Rocky Linux, CentOS):**
  - Package `.spec` file for `sthirayama` and `sthirayama-server`.
  - Automated builds via **Fedora Copr** (`sudo dnf copr enable ... && sudo dnf install sthirayama`).
  - Automatically install and configure systemd service units.
- [ ] **DEB / APT (Ubuntu, Debian):**
  - Debian package archive and Launchpad PPA / GitHub release `.deb`.
  - Pre-configured `systemd` service files and log rotation.

### Multi-Platform & Container Repositories
- [ ] **Homebrew (macOS & Linux):**
  - Tap formula: `brew install chetan/tap/sthirayama`.
  - Supports `brew services start sthirayama`.
- [ ] **OCI / Docker Images:**
  - Multi-architecture container builds (`linux/amd64`, `linux/arm64`).
  - Automated pushes to GitHub Container Registry (`ghcr.io`) and Docker Hub.
- [ ] **One-Line Install Script:**
  - `curl -fsSL https://sthirayama.dev/install.sh | sh`
  - Auto-detects OS/arch, downloads release tarball, and configures `$PATH`.

---

## 3. Client SDKs (Application Libraries)

Allow external programs and applications to integrate with Sthirayama seamlessly:

- [ ] **Python SDK (`sthirayama-py`):**
  - Connection pooling for HTTP/gRPC.
  - Pythonic interface: `client.get("key")`, `client.set("key", "val")`, `client.delete("key")`.
  - Context manager and async client (`asyncio` / `httpx`).
- [ ] **Go SDK (`sthirayama-go`):**
  - Native Go package with retry policies, automatic leader redirection handling, and gRPC client.
- [ ] **Node.js / TypeScript SDK (`sthirayama`):**
  - TypeScript types and Promises for Node, Bun, and Deno environments.
- [ ] **Rust SDK (`sthirayama-client`):**
  - Zero-cost ergonomic client using `reqwest` or `tonic` (gRPC).

---

## 4. Engine Optimizations & Storage Roadmap

Keep performance enhancements focused, measurable, and simple:

```
[Write Path]
Client ──► Memtable (Active) ──► Full ──► Immutable Memtable Queue
                ▲                                   │
               WAL                                  ▼ (Background tokio task)
                                             Flush to L0 SSTable
                                                    │
                                                    ▼ (rayon threadpool)
                                            Compaction (L0 -> L1)
```

### Phase 1: Baseline Benchmarking
- [ ] Measure baseline write and read throughput (IOPS, ops/sec).
- [ ] Measure latency percentiles (p50, p90, p99, p99.9) under concurrent load.
- [ ] Document memory consumption and disk write amplification.

### Phase 2: Asynchronous & Non-Blocking Write Path
- [ ] **Background Memtable Flush:**
  - Split MemTable into active memtable and queue of read-only immutable memtables.
  - When active memtable reaches threshold, swap immediately without blocking writes.
  - Spawn background asynchronous worker to flush immutable memtable to L0 SSTable.
- [ ] **Concurrency with Tokio & Rayon:**
  - Use `tokio` for async I/O scheduling, coordination, and non-blocking gRPC requests.
  - Use `rayon` threadpool for compute-heavy SSTable merges, key comparison pipelines, and chunk compression.

### Phase 3: Read Path Optimizations (Bloom Filters & Caching)
- [ ] **Missing Bloom Filter Implementation:**
  - Generate a Bloom filter block for every SSTable written to disk.
  - Check Bloom filters in memory prior to reading SSTable blocks from disk, eliminating negative lookups (key miss disk reads).
- [ ] **Block Cache (LRU):**
  - In-memory cache of uncompressed SSTable data blocks to serve hot reads directly from RAM.

### Phase 4: Follow-up Benchmarking & Verification
- [ ] Re-run identical benchmark suites against the optimized engine.
- [ ] Compare write stall frequencies, latency under high write loads, and read throughput on cache hits vs misses.

---

## 5. Implementation Milestones

| Milestone | Focus | Deliverables |
| :--- | :--- | :--- |
| **M1: Reliability & Tests** | Engine & Raft Core | Fix remaining test cases, verify snapshot/restore, ensure graceful errors. |
| **M2: Packaging & Docker** | Modes 1 & 2 | Multi-stage Dockerfile, `docker-compose.yml` (3 nodes), single-server daemon launcher. |
| **M3: TUI Development** | Mode 4 | Bubble Tea TUI with live telemetry, dynamic mascot, and key-value management. |
| **M4: Engine Optimizations**| Performance | Baseline benchmark, background flush, tokio/rayon compaction, Bloom filters. |
| **M5: Cloud & Distribution**| Mode 3 & Packaging | k3s / Kubernetes StatefulSet, DNF/Copr RPMs, Homebrew formula, Python/Go SDKs. |
