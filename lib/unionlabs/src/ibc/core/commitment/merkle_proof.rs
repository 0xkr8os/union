use macros::model;
use serde::{Deserialize, Serialize};

use crate::cosmos::ics23::commitment_proof::{CommitmentProof, TryFromCommitmentProofError};
// #[cfg(feature = "ethabi")]
// use crate::InlineFields;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(
    proto(raw(protos::ibc::core::commitment::v1::MerkleProof), into, from),
    ethabi(raw(crate::InlineFields<contracts::glue::IbcCoreCommitmentV1MerkleProofData>), from)
)]
pub struct MerkleProof {
    pub proofs: Vec<CommitmentProof>,
}

#[derive(Debug)]
pub enum TryFromMerkleProofError {
    Proofs(TryFromCommitmentProofError),
}

impl TryFrom<protos::ibc::core::commitment::v1::MerkleProof> for MerkleProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::ibc::core::commitment::v1::MerkleProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proofs: value
                .proofs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromMerkleProofError::Proofs)?,
        })
    }
}

impl From<MerkleProof> for protos::ibc::core::commitment::v1::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MerkleProof> for contracts::glue::IbcCoreCommitmentV1MerkleProofData {
    fn from(value: MerkleProof) -> Self {
        contracts::glue::IbcCoreCommitmentV1MerkleProofData {
            proofs: value.proofs.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MerkleProof>
    for crate::InlineFields<contracts::glue::IbcCoreCommitmentV1MerkleProofData>
{
    fn from(value: MerkleProof) -> Self {
        crate::InlineFields(value.into())
    }
}
