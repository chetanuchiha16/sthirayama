# Sthirayama TUI: Architecture & Visual Design Specification

`sthirayama` combines **Sanskrit** (*Sthira* = Steadfast, Immovable, Enduring) and **Japanese** (*Yama* = Mountain).

This document specifies the Terminal User Interface (TUI) design, visual layout options, the reactive pixel-art mascot, and the color system reflecting its dual architecture: **Rust LSM Engine + Go Raft Consensus**.

---

## 1. Visual Identity & Dual-Language Color Palette

To honor the two-language architecture, the TUI uses **Go Gopher Cyan** for consensus components and **Rust Core Orange** for storage engine components.

| Component / Layer | Role | Primary Color | Hex / Terminal ANSI |
| :--- | :--- | :--- | :--- |
| **Consensus Layer** | Raft Topology, Term, Quorum, Log Replication | **Go Gopher Cyan** | `#00ADD8` / ANSI Cyan |
| **Storage Layer** | Memtable, SSTable Strata, Compaction, WAL | **Rust Core Orange** | `#CE412B` / ANSI Light Red/Orange |
| **Sthira Mascot** | Mountain Peak & System Health Avatar | **Snow White / Stone** | `#E5E9F0` / `#ABB2BF` |
| **Quorum / Success** | Healthy State, Leader Crown | **Emerald Green** | `#48BB78` / ANSI Green |
| **Alert / Danger** | Node Down, Quorum Lost | **Crimson Red** | `#F56565` / ANSI Red |
| **Base Surface** | Backgrounds & Unfocused Borders | **Slate / Charcoal** | `#1E1E2E` / `#2D3748` |

---

## 2. The "Sthira" Dynamic Pixel Mascot

The mascot is an ASCII Mount Fuji whose facial expression reflects cluster health in real time:

```
  Normal / Quorum OK         Compacting / Heavy Load            Node Down / Degraded
        ▲                             ▲                                ▲
       / \                           / \                              / \
      / ˘◡˘ \                       / •̀_•́ \                          / ⊙_⊙ \
     /───────\                     /───────\                        /───────\
    /  = = =  \                   / ⚡ ⚡ ⚡  \                     /  ?   !  \
   /___________ \                /___________ \                   /___________ \
    "Cluster Stable"             "Compacting SSTables..."          "Quorum At Risk!"
```

### Expression States

| Health State | Mascot Face | Trigger Condition | Color Accent |
| :--- | :---: | :--- | :--- |
| **Stable / Quorum** | `( ˘◡˘ )` | All nodes active and quorum healthy (e.g. 3/3). | Emerald Green (`#48BB78`) |
| **Active Compaction** | `( •̀_•́ )` | Memtable flush or SSTable compaction running. | Rust Orange (`#CE412B`) |
| **Leader Election** | `( ◐_◐ )` | Heartbeat lost, candidate voting in progress. | Amber Gold (`#F6AD55`) |
| **Node Down (Warning)**| `( ⊙_⊙ )` | 1 node unreachable, but quorum still holds (2/3). | Amber / Yellow (`#ECC94B`) |
| **Quorum Lost (Split)**| `( ×_× )` | Majority lost (< 2/3). Writes blocked. | Crimson Red (`#F56565`) |

> **Idle Blinking Animation**:
> Using a simple 3-second tick timer, the eyes blink from `( ˘◡˘ )` to `( -◡- )` for 150ms before opening, giving the interface a lively "Tamagotchi" feel.

---

## 3. Layout Options

### Option 1: The Unified Split Dashboard (Default Recommended)

All-in-one split-pane layout showing cluster topology (in Go Cyan), LSM engine (in Rust Orange), live replicated activity, and an interactive shell.

```text
┌─ [▲ STHIRAYAMA] ────────────────────────────────────────── Term: 3 │ QPS: 42 ─┐
│                                                                               │
│ ┌─ Sthira (Mascot) ──┐ ┌─ 1. Raft Topology [Go] ───┐ ┌─ 2. LSM Engine [Rust] ┐ │
│ │        ▲           │ │ Node   Role     Addr   MS │ │ Mem: [████████░░] 2.8MB │ │
│ │       / \          │ │ ● n1 👑Leader  :7001   1  │ │ Keys: 1,420 | Seq: 352  │ │
│ │      / ˘◡˘ \       │ │ ● n2  Follower :7002   2  │ │ WAL: active (seq: 352)  │ │
│ │     /───────\      │ │ ● n3  Follower :7003   1  │ │ ──────────────────────── │ │
│ │    /  = = =  \     │ │                           │ │ Active SSTables (3):     │ │
│ │   /___________ \   │ │ Quorum: 3/3 (Healthy)     │ │ • 000000.sst [a .. h]    │ │
│ │   "Cluster Stable" │ │ Applied: 352 / Term: 3    │ │ • 000001.sst [i .. p]    │ │
│ └────────────────────┘ └───────────────────────────┘ └────────────────────────┘ │
│                                                                               │
│ ┌─ 3. Live Raft Replicated Log Stream ──────────────────────────────────────┐ │
│ │ [18:24:12] #349 SET key="user:98" val="Alice" -> Committed (Quorum 3/3)   │ │
│ │ [18:24:14] #350 SET key="user:99" val="Bob"   -> Committed (Quorum 3/3)   │ │
│ │ [18:24:15] #351 SNAPSHOT Persist triggered -> 104 bytes written to sink   │ │
│ │ [18:24:18] #352 SET key="hello" val="Hina"    -> Committed (Quorum 3/3)   │ │
│ └───────────────────────────────────────────────────────────────────────────┘ │
│                                                                               │
│ ┌─ 4. Interactive Console ──────────────────────────────────────────────────┐ │
│ │ ⛰️ sthirayama > SET user:100 Charlie                                       │ │
│ │ [OK] Committed in 3.2ms. Sequence: 353                                    │ │
│ └───────────────────────────────────────────────────────────────────────────┘ │
└─ [Tab] Switch Node  [c] Compact  [s] Snapshot  [q] Quit ─────────────────────┘
```

---

### Option 2: The Multi-Tab Inspector (k9s style)

Optimized for smaller terminal windows or inspecting deep LSM strata:

* **Tab 1: Overview**: Sthira mascot + node health summary.
* **Tab 2: LSM Storage Inspector**: Memtable skiplist entries, individual SSTable block sizes, index block inspectors, and compaction graphs.
* **Tab 3: Raft Replicated Logs**: Full-screen scrollable log viewer with filter by operation (`SET`, `DEL`, `SNAP`).
* **Tab 4: Interactive Shell**: Full-screen REPL with history and latency metrics.

```text
┌─ [▲ STHIRAYAMA] ── [1: Cluster]  [2: LSM Storage]  [3: Logs]  [4: Shell] ─────┐
│                                                                               │
│   Active View: [2: LSM Storage Inspector - Node 1]                            │
│                                                                               │
│   Memtable Buffer: [██████████████████░░░░░░░░] 2.8 MB / 4.0 MB (70%)         │
│   Write Buffer: 1,420 SkipList Nodes | Active WAL File: ./data-1/wal          │
│                                                                               │
│   SSTable Strata:                                                             │
│   ┌ Level 0 (3 files) ──────────────────────────────────────────────────────┐ │
│   │ 000000.sst │ 1.2 MB │ Keys: [user:0000 .. user:0499] │ 12 Data Blocks   │ │
│   │ 000001.sst │ 1.4 MB │ Keys: [user:0500 .. user:0999] │ 14 Data Blocks   │ │
│   │ 000002.sst │ 1.1 MB │ Keys: [user:1000 .. user:1420] │ 11 Data Blocks   │ │
│   └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                               │
└─ [1-4] Switch Tab  [c] Trigger Compaction  [?] Help  [q] Quit ────────────────┘
```

---

### Option 3: Claude Code / Stream-First Style

Minimalist, scrolling command-line interface with a sticky status badge at the top:

```text
  ⛰️ STHIRAYAMA  v0.1.0  ● 3 nodes  👑 node1 (term 3)
  -------------------------------------------------------------
  [ ˘◡˘ ] All systems nominal. Memtable at 70%.

  • [18:24:12] SET user:98 -> OK (3ms)
  • [18:24:14] SET user:99 -> OK (2ms)
  • [18:24:15] SNAPSHOT created (104 bytes)
  • [18:24:18] SET hello -> OK (1ms)

  ⛰️ sthirayama > _
```

---

## 4. Backend Telemetry Integration (`GET /status`)

To feed live data into the TUI without parsing text logs, `raft/cmd/main.go` will expose a JSON telemetry endpoint:

```http
GET /status
```

### JSON Response Schema
```json
{
  "node_id": "node1",
  "state": "Leader",
  "term": 3,
  "leader_addr": "127.0.0.1:7001",
  "applied_index": 352,
  "commit_index": 352,
  "peers": [
    {"id": "node1", "addr": "127.0.0.1:7001", "role": "Leader", "alive": true, "latency_ms": 0},
    {"id": "node2", "addr": "127.0.0.1:7002", "role": "Follower", "alive": true, "latency_ms": 2},
    {"id": "node3", "addr": "127.0.0.1:7003", "role": "Follower", "alive": true, "latency_ms": 1}
  ],
  "lsm": {
    "memtable_bytes": 2936012,
    "memtable_limit": 4194304,
    "entry_count": 1420,
    "active_sstables": [0, 1, 2],
    "sequence": 352,
    "is_compacting": false
  }
}
```

---

## 5. Technology Stack Recommendation

### Recommended: Go with **Bubble Tea** & **Lip Gloss**
* **Repository location**: `tui/` or `raft/cmd/tui/`
* **Why**:
  - Direct integration with Go's standard HTTP client and your Raft data structures.
  - Bubble Tea's Model-Update-View architecture handles background polling (`tea.Tick`) cleanly.
  - Lip Gloss provides declarative styling matching our hex color palette.
  - Zero cross-compilation friction with the rest of the Go cluster code.

---

## 6. Interactive Controls & Keybindings

| Key | Action | Description |
| :---: | :--- | :--- |
| **`Tab`** | Switch Active Node | Rotates the inspector to view Node 1, 2, or 3 metrics. |
| **`c`** | Trigger Compaction | Sends an RPC to compact active SSTables. |
| **`s`** | Trigger Snapshot | Manually instructs the Raft leader to initiate a snapshot. |
| **`1-4`** | Switch Tab *(in Tabbed view)* | Jumps directly to Cluster, LSM, Logs, or Shell. |
| **`↑ / ↓`** | Command History | Navigates previously executed CLI commands in the prompt. |
| **`Ctrl+C` / `q`** | Quit | Cleanly exits the TUI dashboard. |
