# The Visual Layout Wireframe

```
┌─ Sthirayama Dashboard ────────────────────────────────────── Term: 3 │ QPS: 42 ─┐
│                                                                                 │
│ ┌─ 1. Raft Cluster Topology ──────────┐ ┌─ 2. LSM Engine Internals (Node 1) ─┐  │
│ │ Node     Role       Raft Addr Health│ │ Memtable: [████████░░░░] 2.8KB / 4KB│  │
│ │ ● node1  👑 Leader  :7001     1ms   │ │ Entries:  142 keys | Seq: 352      │  │
│ │ ● node2  Follower   :7002     2ms   │ │ WAL:      active (seq: 352)        │  │
│ │ ● node3  Follower   :7003     1ms   │ │ ────────────────────────────────── │  │
│ │                                     │ │ Active SSTables (3 files):         │  │
│ │ Quorum: 3/3 alive (Healthy)         │ │ • 000000.sst  [user:00 .. user:34] │  │
│ │ Commit Index: 352  Applied: 352     │ │ • 000001.sst  [user:35 .. user:70] │  │
│ └─────────────────────────────────────┘ │ • 000002.sst  [user:71 .. user:99] │  │
│                                         └────────────────────────────────────┘  │
│ ┌─ 3. Live Raft Replicated Log Stream ────────────────────────────────────────┐ │
│ │ [18:24:12] #349 SET key="user:98" val="Alice" -> Committed (Quorum 3/3)     │ │
│ │ [18:24:14] #350 SET key="user:99" val="Bob"   -> Committed (Quorum 3/3)     │ │
│ │ [18:24:15] #351 SNAPSHOT Persist triggered -> 104 bytes written to sink     │ │
│ │ [18:24:18] #352 SET key="hello" val="Hina"    -> Committed (Quorum 3/3)     │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                 │
│ ┌─ 4. Interactive Console ────────────────────────────────────────────────────┐ │
│ │ > SET user:100 Charlie                                                      │ │
│ │ [OK] Committed in 3.2ms. Sequence: 353                                      │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
└─ [Tab] Switch Node  [c] Compact  [s] Trigger Snapshot  [q] Quit ──────────────┘
```

