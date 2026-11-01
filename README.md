# YODPAY Core

**Offline-first payment protocol for partitioned and constrained networks.**

YODPAY is the first product of **Gideon Mesh** — an edge-first architecture designed to keep critical coordination and value transfer alive when central networks and power grids fail.

This repository contains a complete working protocol foundation, including adaptive consensus, cryptographic safety, local mesh abstraction, reconciliation, and supporting tools.

---

## Core Features

- **Adaptive Local Byzantine Consensus**
  - `3f+1` → Normal conditions
  - `2f+1` → Network partition
  - `1f+1` → Extreme isolation (requires human cryptographic approval)

- Offline spending limits with tiered risk controls  
- Double-spend protection via sequential nonces  
- Ed25519 cryptographic signatures  
- Human cryptographic authority in Extreme mode  
- Edge persistence (SQLite)  
- Telemetry-driven network mode switching  
- Local mesh networking abstraction  
- Deterministic reconciliation layer  
- Structured key management  

Transaction lifecycle:  
`PendingOffline → ConfirmedLocal → Settled`

---

## Repository Structure

```text
rust/
├── src/
│   ├── main.rs              # Integrated node implementation
│   ├── crypto.rs            # Ed25519 signatures + human approval
│   ├── persist.rs           # Edge persistence (SQLite)
│   ├── telemetry.rs         # Telemetry parsing + mode influence
│   ├── mesh.rs              # Local mesh networking abstraction
│   ├── reconciliation.rs    # Deterministic online reconciliation
│   └── keystore.rs          # Key management
├── Cargo.toml
cpp/                         # C++ reference implementation
gideon_mesh_simulator.py     # Interactive adaptive consensus simulator
Gideon_Mesh_Contrarian_Thesis_and_Architecture.md
README.md
