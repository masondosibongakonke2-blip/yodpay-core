//! Local Mesh Networking Layer
//! Provides a clean abstraction for peer-to-peer communication.
//! Current implementation is in-memory simulation.
//! Designed so real transports (Bluetooth, Wi-Fi Direct, LoRa, etc.) can be plugged in later.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::Transaction;

#[derive(Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub last_seen: u64,
    pub link_quality: f32, // 0.0 – 1.0
}

pub trait MeshTransport {
    fn broadcast(&self, tx: &Transaction) -> Result<(), String>;
    fn send_to(&self, peer_id: &str, tx: &Transaction) -> Result<(), String>;
    fn discover_peers(&self) -> Vec<PeerInfo>;
}

/// Simple in-memory mesh for development and demonstration
pub struct InMemoryMesh {
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    inbox: Arc<Mutex<Vec<Transaction>>>,
}

impl InMemoryMesh {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
            inbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_peer(&self, peer_id: &str, link_quality: f32) {
        let mut peers = self.peers.lock().unwrap();
        peers.insert(
            peer_id.to_string(),
            PeerInfo {
                peer_id: peer_id.to_string(),
                last_seen: chrono_like_timestamp(),
                link_quality,
            },
        );
    }

    pub fn push_incoming(&self, tx: Transaction) {
        self.inbox.lock().unwrap().push(tx);
    }

    pub fn drain_inbox(&self) -> Vec<Transaction> {
        let mut inbox = self.inbox.lock().unwrap();
        let items = inbox.clone();
        inbox.clear();
        items
    }
}

impl MeshTransport for InMemoryMesh {
    fn broadcast(&self, tx: &Transaction) -> Result<(), String> {
        // In a real implementation this would send over BLE / Wi-Fi Direct / etc.
        println!("[MESH] Broadcasting tx {} to local peers", tx.id);
        Ok(())
    }

    fn send_to(&self, peer_id: &str, tx: &Transaction) -> Result<(), String> {
        println!("[MESH] Sending tx {} to peer {}", tx.id, peer_id);
        Ok(())
    }

    fn discover_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap().values().cloned().collect()
    }
}

fn chrono_like_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
