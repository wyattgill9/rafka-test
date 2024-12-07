use crate::p2p::NodeInfo;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, oneshot};
use rafka_core::{Message, Result, Error};
use std::time::{Duration, SystemTime};

pub struct PartitionTransferManager {
    node_id: String,
    active_transfers: Arc<DashMap<u32, TransferState>>,
    partition_logs: Arc<DashMap<u32, PartitionLog>>,
    transfer_coordinator: Arc<TransferCoordinator>,
}

#[derive(Debug)]
struct TransferState {
    partition_id: u32,
    source_node: NodeInfo,
    target_node: NodeInfo,
    start_offset: u64,
    current_offset: u64,
    status: TransferStatus,
    started_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
enum TransferStatus {
    Preparing,
    Transferring,
    Verifying,
    Completed,
    Failed(String),
}

impl PartitionTransferManager {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            active_transfers: Arc::new(DashMap::new()),
            partition_logs: Arc::new(DashMap::new()),
            transfer_coordinator: Arc::new(TransferCoordinator::new()),
        }
    }

    pub async fn initiate_transfer(
        &self,
        partition_id: u32,
        target_node: NodeInfo,
    ) -> Result<()> {
        // Create transfer state
        let transfer = TransferState {
            partition_id,
            source_node: NodeInfo { id: self.node_id.clone(), ..Default::default() },
            target_node: target_node.clone(),
            start_offset: self.get_partition_offset(partition_id)?,
            current_offset: 0,
            status: TransferStatus::Preparing,
            started_at: SystemTime::now(),
        };

        self.active_transfers.insert(partition_id, transfer);

        // Start transfer process
        self.start_transfer(partition_id, target_node).await
    }

    async fn start_transfer(&self, partition_id: u32, target_node: NodeInfo) -> Result<()> {
        // 1. Prepare transfer
        self.prepare_transfer(partition_id).await?;

        // 2. Stream partition data
        self.stream_partition_data(partition_id, target_node.clone()).await?;

        // 3. Verify transfer
        self.verify_transfer(partition_id, target_node).await?;

        // 4. Complete transfer
        self.complete_transfer(partition_id).await
    }

    async fn prepare_transfer(&self, partition_id: u32) -> Result<()> {
        let mut transfer = self.active_transfers.get_mut(&partition_id)
            .ok_or_else(|| Error::Transfer("Transfer not found".into()))?;
        
        // Pause writes to partition
        self.transfer_coordinator.pause_partition(partition_id).await?;
        
        transfer.status = TransferStatus::Transferring;
        Ok(())
    }

    async fn stream_partition_data(
        &self,
        partition_id: u32,
        target_node: NodeInfo,
    ) -> Result<()> {
        let log = self.partition_logs.get(&partition_id)
            .ok_or_else(|| Error::Transfer("Partition log not found".into()))?;

        let batch_size = 1000;
        let mut offset = log.start_offset;

        while offset < log.current_offset {
            let messages = log.get_messages(offset, batch_size)?;
            
            // Send batch to target node
            self.send_batch_to_target(&target_node, partition_id, offset, &messages).await?;
            
            // Update transfer state
            if let Some(mut transfer) = self.active_transfers.get_mut(&partition_id) {
                transfer.current_offset = offset;
            }

            offset += messages.len() as u64;
        }

        Ok(())
    }

    async fn verify_transfer(
        &self,
        partition_id: u32,
        target_node: NodeInfo,
    ) -> Result<()> {
        let mut transfer = self.active_transfers.get_mut(&partition_id)
            .ok_or_else(|| Error::Transfer("Transfer not found".into()))?;
        
        transfer.status = TransferStatus::Verifying;

        // Verify data integrity
        let source_hash = self.compute_partition_hash(partition_id).await?;
        let target_hash = self.request_partition_hash(&target_node, partition_id).await?;

        if source_hash != target_hash {
            transfer.status = TransferStatus::Failed("Hash mismatch".into());
            return Err(Error::Transfer("Transfer verification failed".into()));
        }

        Ok(())
    }

    async fn complete_transfer(&self, partition_id: u32) -> Result<()> {
        let mut transfer = self.active_transfers.get_mut(&partition_id)
            .ok_or_else(|| Error::Transfer("Transfer not found".into()))?;
        
        // Update cluster metadata
        self.transfer_coordinator.update_partition_ownership(
            partition_id,
            transfer.target_node.clone(),
        ).await?;

        // Resume writes to partition on new owner
        self.transfer_coordinator.resume_partition(partition_id).await?;

        transfer.status = TransferStatus::Completed;
        Ok(())
    }
} 