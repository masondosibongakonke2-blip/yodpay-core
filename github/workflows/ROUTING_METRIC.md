```mermaid
graph TB
    %% Structural Subgraphs for Layer Isolation
    subgraph Space_Layer [1. HIGH-FRONTIER WAN TELEMETRY LAYER]
        direction LR
        Starlink_Sat1[Starlink Array Master Node] <--> |Laser Inter-Satellite Link| Starlink_Sat2[Starlink Orbital Relay]
    end

    subgraph Root_Infrastructure [2. GLOBAL FINANCIAL LOGISTICS LAYER]
        direction TB
        Root_Engine[Gideon Mesh Root Settlement Engine]
        Bank_Rail[Legacy ISO 20022 Core Banking Rails]
        Root_Engine <--> |Synchronous Ledger Bridge| Bank_Rail
    end

    subgraph Partitioned_Edge [3. LOCAL SYSTEMIC ISOLATION BOUNDARY - EMERGENCY MESH]
        direction TR
        
        %% Edge Nodes
        Node_Alpha[YODPAY Edge Node A: Master POS]
        Node_Beta[YODPAY Edge Node B: Mobile Wallet Client]
        Node_Gamma[YODPAY Edge Node C: Embedded Microcontroller]
        Node_Delta[YODPAY Edge Node D: Local Hardware Gateway]

        %% Local Ad-Hoc Mesh Interconnects
        Node_Alpha <--> |Ad-Hoc Wi-Fi Peer Comm| Node_Beta
        Node_Beta <--> |Low-Energy Bluetooth Mesh| Node_Gamma
        Node_Gamma <--> |Ad-Hoc Wi-Fi Peer Comm| Node_Delta
        Node_Delta <--> |Local Radio Frequency Rail| Node_Alpha
        
        %% Local consensus loop visualization
        Node_Alpha & Node_Beta & Node_Gamma & Node_Delta --> |Local Consensus Vote Evaluator| Quorum_Engine{Adaptive BFT Engine<br>Calculates f Nodes}
        Quorum_Engine -->|3f+1 / 2f+1 Quorum Met| Finality_State(State: ConfirmedLocal<br>Value Floating / Locked at Edge)
    end

    %% Asynchronous Physical Reconnection Connections
    Starlink_Sat1 <--> |Intermittent Telemetry Teleport Link| Root_Infrastructure
    Node_Delta -.-> |Asynchronous Backhaul Path When Available| Starlink_Sat2

    %% Styling and Structural Visual Aesthetics
    style Space_Layer fill:#111b24,stroke:#38bdf8,stroke-width:2px,color:#fff
    style Root_Infrastructure fill:#112211,stroke:#22c55e,stroke-width:2px,color:#fff
    style Partitioned_Edge fill:#1c1917,stroke:#ea580c,stroke-width:2px,color:#fff
    style Quorum_Engine fill:#7c3aed,stroke:#a78bfa,stroke-width:1px,color:#fff
    style Finality_State fill:#15171c,stroke:#facc15,stroke-width:2px,color:#fff
```
