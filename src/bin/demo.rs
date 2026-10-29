use std::collections::HashMap;
use std::thread;
use std::time::Duration;

// --- Simplified Mirror of your Core Module for Simulation Self-Containment ---
#[derive(Debug, Clone)]
pub struct Tx {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone)]
pub enum Topology {
    Normal,
    Partitioned,
    Extreme,
}

pub struct SimulationNode {
    pub node_id: String,
    pub balance: u64,
    pub offline_limit: u64,
    pub spent_offline: u64,
    pub nonce: u64,
    pub spent_nonces: std::collections::HashSet<u64>,
}

impl SimulationNode {
    pub fn new(id: &str, bal: u64, lim: u64) -> Self {
        Self {
            node_id: id.to_string(),
            balance: bal,
            offline_limit: lim,
            spent_offline: 0,
            nonce: 0,
            spent_nonces: std::collections::HashSet::new(),
        }
    }
}

// --- Visual Printing Helper Functions ---
fn print_header(title: &str) {
    println!("\n================================================================================");
    println!(">>> SYSTEM EVENT: {}", title.to_uppercase());
    println!("================================================================================");
}

fn sleep_step() {
    thread::sleep(Duration::from_millis(1200));
}

fn main() {
    // -------------------------------------------------------------------------
    // SETUP PHASE
    // -------------------------------------------------------------------------
    print_header("Initializing Local Gideon Mesh Infrastructure");
    println!("[INIT] Initializing 4 active local edge consensus nodes...");
    
    let mut mesh: HashMap<String, SimulationNode> = HashMap::new();
    mesh.insert("Node_Alpha".to_string(), SimulationNode::new("Node_Alpha", 1000, 400));
    mesh.insert("Node_Beta".to_string(),  SimulationNode::new("Node_Beta", 500, 200));
    mesh.insert("Node_Gamma".to_string(), SimulationNode::new("Node_Gamma", 500, 200));
    mesh.insert("Node_Delta".to_string(), SimulationNode::new("Node_Delta", 500, 200));

    println!("[MESH] Node_Alpha spawned | Balance: 1000 | Offline Credit Window: 400");
    println!("[MESH] Nodes Beta, Gamma, Delta spawned online as validation peers.");
    println!("[STATUS] Global Root Ledger: CONNECTED (Latency: 12ms)");
    sleep_step();

    // -------------------------------------------------------------------------
    // DISCONNECTION EVENT
    // -------------------------------------------------------------------------
    print_header("Network Partition Triggered (Load-Shedding / Outage)");
    println!("⚠️ [ALERT] WAN Backhaul disconnected from Central Root Ledger!");
    println!("⚠️ [ALERT] Local cluster enters Autonomous Offline Mode.");
    println!("[ENGINE] Scanning active local topology connectivity graph...");
    
    let active_peers = 4;
    // f = (4 - 1) / 3 = 1
    let topology = Topology::Partitioned; 
    println!("[CONSENSUS] 4 peers reachable locally. Byzantine threshold (f) = 1.");
    println!("[CONSENSUS] Switching routing matrix parameters to: PARTITIONED MODE (2f + 1)");
    sleep_step();

    // -------------------------------------------------------------------------
    // TRANSACTION INITIALIZATION
    // -------------------------------------------------------------------------
    print_header("Offline Payment Execution at Edge Boundary");
    println!("[TX_INIT] Alice initiates off-grid payment via Node_Alpha to Bob.");
    println!("[TX_INIT] Payload Details: From: Alice, To: Bob, Amount: 250");
    
    let alpha = mesh.get_mut("Node_Alpha").unwrap();
    
    // Safety check allocations natively
    if alpha.spent_offline + 250 > alpha.offline_limit {
        println!("❌ [ERROR] Transaction aborted: Offline Limit Exceeded.");
        return;
    }
    
    // Mutate state locally
    alpha.spent_nonces.insert(alpha.nonce);
    let signed_tx = Tx {
        id: format!("tx_alice_{}", alpha.nonce),
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 250,
        nonce: alpha.nonce,
    };
    
    alpha.balance -= 250;
    alpha.spent_offline += 250;
    alpha.nonce += 1;

    println!("✅ [LOCAL_MUTATION] Node_Alpha updated internal ledger safely.");
    println!("   -> Alice Local Balance: 750 (Debited immediately)");
    println!("   -> Cumulative Offline Spent Quota: 250 / 400");
    println!("   -> Next Expected Nonce: {}", alpha.nonce);
    sleep_step();

    // -------------------------------------------------------------------------
    // MESH QUORUM BROADCAST
    // -------------------------------------------------------------------------
    print_header("Localized Peer-to-Peer Consensus Evaluation");
    println!("[BROADCAST] Broadcasting Payload '{}' across ad-hoc Wi-Fi/Bluetooth mesh...", signed_tx.id);
    
    // Simulate quorum voting
    let mut approvals = 1; // Node_Alpha itself
    println!("[VOTE] Peer Node_Beta validates cryptographic signatures: APPROVED"); approvals += 1;
    println!("[VOTE] Peer Node_Gamma validates structural nonces: APPROVED"); approvals += 1;
    println!("[VOTE] Peer Node_Delta is temporarily offline/latent: NO VOTE");
    
    println!("[EVALUATION] Total Approvals Received: {} / Required (2f+1): 3", approvals);
    if approvals >= 3 {
        println!("⚖️ [FINALITY] Quorum constraints satisfied! Transaction status -> CONFIRMEDLOCAL");
        println!("🔒 [LOCAL_LEDGER] Transaction sealed in local mesh journals. Safe for off-grid handoff.");
    }
    sleep_step();

    // -------------------------------------------------------------------------
    // SECURITY BLOCK DEMONSTRATION (DOUBLE SPEND VECTOR)
    // -------------------------------------------------------------------------
    print_header("Malicious Threat Vector Interception (Double-Spend Attack)");
    println!("👿 [ATTACK] Malicious actor manipulates software to replay Alice Nonce: 0");
    println!("👿 [ATTACK] Replayed Payload Details: From: Alice, To: Charlie, Amount: 100, Nonce: 0");
    
    let attack_nonce = 0;
    println!("[ENGINE] Node_Alpha intercepting payload stream. Checking nonce freshness mapping...");
    
    if alpha.spent_nonces.contains(&attack_nonce) {
        println!("🛡️ [SECURITY ALERT] NONCE COLLISION DETECTED FOR SENDER 'ALICE' AT NONCE: {}!", attack_nonce);
        println!("🛑 [REJECTION] Transaction footprint matched signatures in 'spent_nonces' cache.");
        println!("🛡️ [COUNTERMEASURE] Transaction aborted instantly. Zero memory leak. System integrity secured.");
    }
    sleep_step();

    // -------------------------------------------------------------------------
    // ROOT RECONCILIATION PHASE
    // -------------------------------------------------------------------------
    print_header("WAN Uplink Restored & Global Root Ledger Reconciliation");
    println!("📡 [TELEMETRY] Starlink Backhaul Connection Re-established with Root Network!");
    println!("[RECONCILE] Shipping local buffer queue sequentially to central cloud platform...");
    println!("[IDEMPOTENCY] Root Gateway verifying globally registered transaction hashes...");
    println!("[IDEMPOTENCY] Identity token '{}' verified as UNIQUE. Proceeding to settle allocations.", signed_tx.id);
    
    // Complete global settlement changes
    println!("💰 [ROOT_MUTATION] Mutating centralized global database structures:");
    println!("   -> Bob Account Credited permanently: +250");
    
    // Liberate quotas back on the local node boundary
    let alpha_sync = mesh.get_mut("Node_Alpha").unwrap();
    alpha_sync.spent_offline -= 250; // Quota released!
    
    println!("🔄 [QUORUM_RESET] Pushing clearing signal feedback loop down to edge node boundary...");
    println!("✅ [SUCCESS] Node_Alpha reset temporary tracking quota.");
    println!("   -> Alice Restored Offline Credit Window Availability: {} / 400", alpha_sync.spent_offline);
    
    print_header("Simulation Successfully Terminated - Protocol Functioning Within Bounds");
}
