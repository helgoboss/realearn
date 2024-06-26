use crate::domain::InstanceId;
use crate::infrastructure::proto::{
    event_reply, ContinuousColumnUpdate, ContinuousMatrixUpdate, EventReply,
    OccasionalGlobalUpdate, OccasionalInstanceUpdate, OccasionalMatrixUpdate,
    OccasionalPlaytimeEngineUpdate, QualifiedContinuousSlotUpdate, QualifiedOccasionalClipUpdate,
    QualifiedOccasionalColumnUpdate, QualifiedOccasionalRowUpdate, QualifiedOccasionalSlotUpdate,
    QualifiedOccasionalTrackUpdate, QualifiedOccasionalUnitUpdate,
};
use futures::future;
use tokio::sync::broadcast::{Receiver, Sender};

/// This must be a global object because it's responsible for supplying one gRPC endpoint with
/// streaming data, and we have only one endpoint for all matrices.
#[derive(Clone, Debug)]
pub struct ProtoSenders {
    pub occasional_global_update_sender: Sender<OccasionalGlobalUpdateBatch>,
    pub occasional_instance_update_sender: Sender<OccasionalInstanceUpdateBatch>,
    pub occasional_unit_update_sender: Sender<OccasionalUnitUpdateBatch>,
    pub occasional_playtime_engine_update_sender: Sender<OccasionalPlaytimeEngineUpdateBatch>,
    pub occasional_matrix_update_sender: Sender<OccasionalMatrixUpdateBatch>,
    pub occasional_track_update_sender: Sender<OccasionalTrackUpdateBatch>,
    pub occasional_column_update_sender: Sender<OccasionalColumnUpdateBatch>,
    pub occasional_row_update_sender: Sender<OccasionalRowUpdateBatch>,
    pub occasional_slot_update_sender: Sender<OccasionalSlotUpdateBatch>,
    pub occasional_clip_update_sender: Sender<OccasionalClipUpdateBatch>,
    pub continuous_matrix_update_sender: Sender<ContinuousMatrixUpdateBatch>,
    pub continuous_column_update_sender: Sender<ContinuousColumnUpdateBatch>,
    pub continuous_slot_update_sender: Sender<ContinuousSlotUpdateBatch>,
}

#[derive(Debug)]
pub struct ProtoReceivers {
    pub occasional_global_update_receiver: Receiver<OccasionalGlobalUpdateBatch>,
    pub occasional_instance_update_receiver: Receiver<OccasionalInstanceUpdateBatch>,
    pub occasional_unit_update_receiver: Receiver<OccasionalUnitUpdateBatch>,
    pub occasional_playtime_engine_update_receiver: Receiver<OccasionalPlaytimeEngineUpdateBatch>,
    pub occasional_matrix_update_receiver: Receiver<OccasionalMatrixUpdateBatch>,
    pub occasional_track_update_receiver: Receiver<OccasionalTrackUpdateBatch>,
    pub occasional_column_update_receiver: Receiver<OccasionalColumnUpdateBatch>,
    pub occasional_row_update_receiver: Receiver<OccasionalRowUpdateBatch>,
    pub occasional_slot_update_receiver: Receiver<OccasionalSlotUpdateBatch>,
    pub occasional_clip_update_receiver: Receiver<OccasionalClipUpdateBatch>,
    pub continuous_matrix_update_receiver: Receiver<ContinuousMatrixUpdateBatch>,
    pub continuous_column_update_receiver: Receiver<ContinuousColumnUpdateBatch>,
    pub continuous_slot_update_receiver: Receiver<ContinuousSlotUpdateBatch>,
}

impl ProtoReceivers {
    pub async fn keep_processing_updates(
        &mut self,
        instance_id: InstanceId,
        process: &impl Fn(EventReply),
    ) {
        future::join5(
            future::join5(
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.continuous_matrix_update_receiver,
                ),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.continuous_column_update_receiver,
                ),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.continuous_slot_update_receiver,
                ),
                keep_processing_updates(process, &mut self.occasional_global_update_receiver),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.occasional_instance_update_receiver,
                ),
            ),
            future::join5(
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.occasional_matrix_update_receiver,
                ),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.occasional_track_update_receiver,
                ),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.occasional_column_update_receiver,
                ),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.occasional_row_update_receiver,
                ),
                keep_processing_session_filtered_updates(
                    instance_id,
                    process,
                    &mut self.occasional_slot_update_receiver,
                ),
            ),
            keep_processing_session_filtered_updates(
                instance_id,
                process,
                &mut self.occasional_clip_update_receiver,
            ),
            keep_processing_session_filtered_updates(
                instance_id,
                process,
                &mut self.occasional_unit_update_receiver,
            ),
            keep_processing_updates(
                process,
                &mut self.occasional_playtime_engine_update_receiver,
            ),
        )
        .await;
    }
}

async fn keep_processing_session_filtered_updates<T>(
    instance_id: InstanceId,
    process: impl Fn(EventReply),
    receiver: &mut Receiver<WithInstanceId<T>>,
) where
    T: Clone + Into<event_reply::Value>,
{
    loop {
        if let Ok(batch) = receiver.recv().await {
            if batch.instance_id != instance_id {
                continue;
            }
            let reply = EventReply {
                value: Some(batch.value.into()),
            };
            process(reply);
        }
    }
}

async fn keep_processing_updates<T>(process: impl Fn(EventReply), receiver: &mut Receiver<T>)
where
    T: Clone + Into<event_reply::Value>,
{
    loop {
        if let Ok(batch) = receiver.recv().await {
            let reply = EventReply {
                value: Some(batch.into()),
            };
            process(reply);
        }
    }
}

impl Default for ProtoSenders {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtoSenders {
    pub fn new() -> Self {
        Self {
            occasional_global_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_instance_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_unit_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_playtime_engine_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_matrix_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_track_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_column_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_row_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_slot_update_sender: tokio::sync::broadcast::channel(100).0,
            occasional_clip_update_sender: tokio::sync::broadcast::channel(100).0,
            continuous_slot_update_sender: tokio::sync::broadcast::channel(1000).0,
            continuous_column_update_sender: tokio::sync::broadcast::channel(500).0,
            continuous_matrix_update_sender: tokio::sync::broadcast::channel(500).0,
        }
    }

    pub fn subscribe_to_all(&self) -> ProtoReceivers {
        ProtoReceivers {
            occasional_global_update_receiver: self.occasional_global_update_sender.subscribe(),
            occasional_instance_update_receiver: self.occasional_instance_update_sender.subscribe(),
            occasional_unit_update_receiver: self.occasional_unit_update_sender.subscribe(),
            occasional_playtime_engine_update_receiver: self
                .occasional_playtime_engine_update_sender
                .subscribe(),
            occasional_matrix_update_receiver: self.occasional_matrix_update_sender.subscribe(),
            occasional_track_update_receiver: self.occasional_track_update_sender.subscribe(),
            occasional_column_update_receiver: self.occasional_column_update_sender.subscribe(),
            occasional_row_update_receiver: self.occasional_row_update_sender.subscribe(),
            occasional_slot_update_receiver: self.occasional_slot_update_sender.subscribe(),
            occasional_clip_update_receiver: self.occasional_clip_update_sender.subscribe(),
            continuous_matrix_update_receiver: self.continuous_matrix_update_sender.subscribe(),
            continuous_column_update_receiver: self.continuous_column_update_sender.subscribe(),
            continuous_slot_update_receiver: self.continuous_slot_update_sender.subscribe(),
        }
    }
}

#[derive(Clone)]
pub struct WithInstanceId<T> {
    pub instance_id: InstanceId,
    pub value: T,
}

pub type OccasionalGlobalUpdateBatch = Vec<OccasionalGlobalUpdate>;
pub type OccasionalInstanceUpdateBatch = WithInstanceId<Vec<OccasionalInstanceUpdate>>;
pub type OccasionalUnitUpdateBatch = WithInstanceId<Vec<QualifiedOccasionalUnitUpdate>>;
pub type OccasionalPlaytimeEngineUpdateBatch = Vec<OccasionalPlaytimeEngineUpdate>;
pub type OccasionalMatrixUpdateBatch = WithInstanceId<Vec<OccasionalMatrixUpdate>>;
pub type OccasionalTrackUpdateBatch = WithInstanceId<Vec<QualifiedOccasionalTrackUpdate>>;
pub type OccasionalColumnUpdateBatch = WithInstanceId<Vec<QualifiedOccasionalColumnUpdate>>;
pub type OccasionalRowUpdateBatch = WithInstanceId<Vec<QualifiedOccasionalRowUpdate>>;
pub type OccasionalSlotUpdateBatch = WithInstanceId<Vec<QualifiedOccasionalSlotUpdate>>;
pub type OccasionalClipUpdateBatch = WithInstanceId<Vec<QualifiedOccasionalClipUpdate>>;
pub type ContinuousMatrixUpdateBatch = WithInstanceId<ContinuousMatrixUpdate>;
pub type ContinuousColumnUpdateBatch = WithInstanceId<Vec<ContinuousColumnUpdate>>;
pub type ContinuousSlotUpdateBatch = WithInstanceId<Vec<QualifiedContinuousSlotUpdate>>;
