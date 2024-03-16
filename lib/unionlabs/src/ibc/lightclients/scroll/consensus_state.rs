use macros::model;

use crate::{errors::InvalidLength, hash::H256};

#[model(proto(
    raw(protos::union::ibc::lightclients::scroll::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    pub batch_index: u64,
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}

impl From<ConsensusState> for protos::union::ibc::lightclients::scroll::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            batch_index: value.batch_index,
            ibc_storage_root: value.ibc_storage_root.into(),
            timestamp: value.timestamp,
        }
    }
}

#[derive(Debug)]
pub enum TryFromConsensusStateError {
    IbcStorageRoot(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            batch_index: value.batch_index,
            ibc_storage_root: value
                .ibc_storage_root
                .try_into()
                .map_err(TryFromConsensusStateError::IbcStorageRoot)?,
            timestamp: value.timestamp,
        })
    }
}
