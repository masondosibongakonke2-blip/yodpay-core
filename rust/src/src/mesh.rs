//! Mesh transport interface.
//! Real implementations (BLE / Wi-Fi Direct / LoRa) plug in here later.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMessage {
    pub from_node: String,
    pub kind: String,          // "confirm" | "tx_announce" | "reconcile"
    pub payload: Vec<u8>,
}

#[async_trait]
pub trait MeshTransport: Send + Sync {
    async fn broadcast(&self, msg: MeshMessage) -> Result<(), String>;
    async fn recv(&self) -> Result<MeshMessage, String>;
    fn local_node_id(&self) -> &str;
}

/// In-memory simulated transport (current behaviour)
pub struct SimulatedMesh {
    pub node_id: String,
}

#[async_trait]
impl MeshTransport for SimulatedMesh {
    async fn broadcast(&self, _msg: MeshMessage) -> Result<(), String> {
        // TODO: replace with BLE / Wi-Fi Direct / LoRa broadcast
        Ok(())
    }
    async fn recv(&self) -> Result<MeshMessage, String> {
        Err("simulated: no message".into())
    }
    fn local_node_id(&self) -> &str {
        &self.node_id
    }
}
