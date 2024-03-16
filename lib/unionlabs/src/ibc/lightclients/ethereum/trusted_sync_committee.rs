use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use macros::model;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use tree_hash::TreeHash;

use crate::{
    errors::{required, MissingField},
    ethereum::config::SYNC_COMMITTEE_SIZE,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::sync_committee::{SyncCommittee, TryFromSyncCommitteeError},
    },
};

/// Sync committee that is going to be used to verify the update
///
/// Note that the verifier uses one of them based on whether the signature slot
/// is equal to the current slot or current slot + 1
#[derive(
    CloneNoBound, DebugNoBound, PartialEqNoBound, Encode, Decode, TreeHash, Serialize, Deserialize,
)]
#[ssz(enum_behaviour = "union")]
#[tree_hash(enum_behaviour = "union")]
#[serde(
    tag = "@type",
    content = "@value",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields,
    rename_all = "snake_case"
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum ActiveSyncCommittee<C: SYNC_COMMITTEE_SIZE> {
    Current(SyncCommittee<C>),
    Next(SyncCommittee<C>),
}

impl<C: SYNC_COMMITTEE_SIZE> ActiveSyncCommittee<C> {
    #[must_use]
    pub fn get(&self) -> &SyncCommittee<C> {
        match self {
            ActiveSyncCommittee::Current(committee) | ActiveSyncCommittee::Next(committee) => {
                committee
            }
        }
    }

    #[must_use]
    pub fn get_mut(&mut self) -> &mut SyncCommittee<C> {
        match self {
            ActiveSyncCommittee::Current(committee) | ActiveSyncCommittee::Next(committee) => {
                committee
            }
        }
    }
}

#[derive(
    CloneNoBound, DebugNoBound, PartialEqNoBound, Encode, Decode, TreeHash, Serialize, Deserialize,
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee),
    into,
    from
))]
pub struct TrustedSyncCommittee<C: SYNC_COMMITTEE_SIZE> {
    pub trusted_height: Height,
    pub sync_committee: ActiveSyncCommittee<C>,
}

impl<C: SYNC_COMMITTEE_SIZE> From<TrustedSyncCommittee<C>>
    for protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee
{
    fn from(value: TrustedSyncCommittee<C>) -> Self {
        match value.sync_committee {
            ActiveSyncCommittee::Current(committee) => Self {
                trusted_height: Some(value.trusted_height.into()),
                current_sync_committee: Some(committee.into()),
                next_sync_committee: None,
            },
            ActiveSyncCommittee::Next(committee) => Self {
                trusted_height: Some(value.trusted_height.into()),
                current_sync_committee: None,
                next_sync_committee: Some(committee.into()),
            },
        }
    }
}

#[derive(DebugNoBound)]
pub enum TryFromTrustedSyncCommitteeError {
    MissingField(MissingField),
    SyncCommittee(TryFromSyncCommitteeError),
}

impl<C: SYNC_COMMITTEE_SIZE>
    TryFrom<protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee>
    for TrustedSyncCommittee<C>
{
    type Error = TryFromTrustedSyncCommitteeError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::TrustedSyncCommittee,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            trusted_height: required!(value.trusted_height)?.into(),
            sync_committee: match (value.current_sync_committee, value.next_sync_committee) {
                (None, None) => {
                    return Err(TryFromTrustedSyncCommitteeError::MissingField(
                        MissingField("no current nor next sync committee"),
                    ))
                }
                (None, Some(next_committee)) => ActiveSyncCommittee::Next(
                    next_committee
                        .try_into()
                        .map_err(TryFromTrustedSyncCommitteeError::SyncCommittee)?,
                ),
                (Some(current_committee), _) => ActiveSyncCommittee::Current(
                    current_committee
                        .try_into()
                        .map_err(TryFromTrustedSyncCommitteeError::SyncCommittee)?,
                ),
            },
        })
    }
}
