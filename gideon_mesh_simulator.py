#!/usr/bin/env python3
"""
Gideon Mesh – Adaptive Consensus Interactive Simulator
-------------------------------------------------------
Demonstrates the core innovation of Gideon Mesh:
Dynamic shifting of Byzantine quorum requirements based on network stress.

Modes:
- 3f+1 : Normal (stable connectivity)
- 2f+1 : Partitioned (regional isolation)
- 1f+1 : Extreme (single node + human cryptographic approval)

This simulator is designed so reviewers can trigger infrastructure failure
and watch the protocol adapt in real time.
"""

import time
import random
import hashlib
from typing import List, Dict, Optional
from dataclasses import dataclass, field


# ============================================================
# CONFIGURATION
# ============================================================

LATENCY_NORMAL = 80.0          # ms
LATENCY_PARTITIONED = 280.0    # ms
LATENCY_EXTREME = 700.0        # ms


# ============================================================
# DATA STRUCTURES
# ============================================================

@dataclass
class Transaction:
    tx_id: str
    tx_type: str
    description: str
    amount: float = 0.0
    signatures: Dict[str, str] = field(default_factory=dict)
    status: str = "Pending"

    def sign(self, node_id: str):
        """Simulate cryptographic signature"""
        content = f"{self.tx_id}:{self.description}:{node_id}"
        sig = hashlib.sha256(content.encode()).hexdigest()[:12]
        self.signatures[node_id] = sig


# ============================================================
# NODE
# ============================================================

class MeshNode:
    def __init__(self, node_id: str, region: str, is_human_authority: bool = False):
        self.node_id = node_id
        self.region = region
        self.is_human_authority = is_human_authority
        self.mode = "3f+1 (Normal)"
        self.local_ledger: Dict[str, Transaction] = {}

    def update_mode(self, latency_ms: float):
        if latency_ms >= LATENCY_EXTREME:
            self.mode = "1f+1 (Extreme Isolation)"
        elif latency_ms >= LATENCY_PARTITIONED:
            self.mode = "2f+1 (Partitioned)"
        else:
            self.mode = "3f+1 (Normal)"

    def process_transaction(self, tx: Transaction, all_nodes: List['MeshNode'], latency_ms: float) -> tuple[bool, str]:
        self.update_mode(latency_ms)
        tx.sign(self.node_id)

        # ---------- 3f+1 Normal ----------
        if "3f+1" in self.mode:
            required = max(2, int(len(all_nodes) * 0.67))
            for node in all_nodes:
                if node.node_id != self.node_id and random.random() > 0.1:
                    tx.sign(node.node_id)

            if len(tx.signatures) >= required:
                tx.status = "ConfirmedLocal"
                self.local_ledger[tx.tx_id] = tx
                return True, f"Confirmed under 3f+1. Signatures: {len(tx.signatures)}/{required}"
            return False, f"Failed 3f+1. Only {len(tx.signatures)}/{required} signatures."

        # ---------- 2f+1 Partitioned ----------
        elif "2f+1" in self.mode:
            regional = [n for n in all_nodes if n.region == self.region]
            required = max(1, int(len(regional) * 0.6))
            for node in regional:
                if node.node_id != self.node_id:
                    tx.sign(node.node_id)

            if len(tx.signatures) >= required:
                tx.status = "ConfirmedLocal"
                self.local_ledger[tx.tx_id] = tx
                return True, f"Confirmed under 2f+1 (regional). Signatures: {len(tx.signatures)}/{required}"
            return False, f"Failed 2f+1. Regional signatures insufficient."

        # ---------- 1f+1 Extreme ----------
        else:
            if self.is_human_authority:
                tx.sign("HUMAN_CRYPTOGRAPHIC_KEY")
                tx.status = "ConfirmedLocal"
                self.local_ledger[tx.tx_id] = tx
                return True, "Confirmed under 1f+1. Local node + Human cryptographic approval."
            return False, "Failed 1f+1. Human cryptographic authority required but missing."


# ============================================================
# INTERACTIVE SANDBOX
# ============================================================

def run_simulator():
    print("=" * 70)
    print("  GIDEON MESH – Adaptive Consensus Simulator")
    print("  Dynamic Byzantine Quorum: 3f+1 → 2f+1 → 1f+1")
    print("=" * 70)

    nodes = [
        MeshNode("Node-ZA-01", "Gauteng", is_human_authority=True),
        MeshNode("Node-ZA-02", "Gauteng"),
        MeshNode("Node-ZA-03", "Gauteng"),
        MeshNode("Node-EC-01", "EasternCape", is_human_authority=True),
        MeshNode("Node-EC-02", "EasternCape"),
        MeshNode("Node-EC-03", "EasternCape"),
    ]

    tx_counter = 1001

    while True:
        print("\nCurrent Infrastructure Conditions:")
        print("  [1] Stable Network          (Normal – 3f+1)")
        print("  [2] Regional Degradation    (Partitioned – 2f+1)")
        print("  [3] Total Grid Collapse     (Extreme – 1f+1)")
        print("  [4] Exit")

        choice = input("\nSelect condition (1-4): ").strip()

        if choice == "4":
            print("\nSimulator terminated.")
            break

        if choice == "1":
            latency = random.uniform(40, 90)
            condition = "Stable"
        elif choice == "2":
            latency = random.uniform(220, 350)
            condition = "Regional Degradation"
        elif choice == "3":
            latency = random.uniform(650, 900)
            condition = "Total Grid Collapse"
        else:
            print("Invalid choice.")
            continue

        print(f"\n>>> Injecting condition: {condition} | Latency: {latency:.1f}ms")

        tx = Transaction(
            tx_id=f"TX-{tx_counter}",
            tx_type="PAYMENT",
            description="Local merchant payment during infrastructure stress",
            amount=1500.0
        )
        tx_counter += 1

        gateway = nodes[0]  # Node with human authority
        print(f"Processing {tx.tx_id} via {gateway.node_id}...")
        time.sleep(0.6)

        success, message = gateway.process_transaction(tx, nodes, latency)

        print("-" * 60)
        print(f"Result     : {'SUCCESS' if success else 'FAILED'}")
        print(f"Active Mode: {gateway.mode}")
        print(f"Detail     : {message}")
        print(f"Signatures : {list(tx.signatures.keys())}")
        print("-" * 60)

        input("\nPress Enter to continue...")


if __name__ == "__main__":
    run_simulator()
