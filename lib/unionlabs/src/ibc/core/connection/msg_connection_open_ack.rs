use core::fmt::Debug;

use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{client::height::Height, connection::version::Version},
    id::ConnectionId,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::ibc::core::connection::v1::MsgConnectionOpenAck)]
pub struct MsgConnectionOpenAck<ClientState, ProofTry, ProofClient, ProofConsensus> {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub version: Version,
    pub client_state: ClientState,
    // TODO: Make this type generic
    pub proof_height: Height,
    pub proof_try: ProofTry,
    pub proof_client: ProofClient,
    pub proof_consensus: ProofConsensus,
    // TODO: Make this type generic
    pub consensus_height: Height,
}
