use std::string::FromUtf8Error;

use cosmwasm_std::{IbcOrder, StdError, SubMsgResult};
use cw_controllers::AdminError;
use thiserror::Error;
use ucs01_relay_api::{protocol::ProtocolError, types::EncodingError};

/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Channel doesn't exist: {id}")]
    NoSuchChannel { id: String },

    #[error("Didn't send any funds")]
    NoFunds,

    #[error("Expected {expected:?} channel ordering but got {actual:?}")]
    InvalidChannelOrdering {
        expected: IbcOrder,
        actual: IbcOrder,
    },

    #[error("Insufficient funds to redeem on channel")]
    InsufficientFunds,

    #[error("Got a submessage reply with unknown id: {id} and variant: {variant:?}")]
    UnknownReply { id: u64, variant: SubMsgResult },

    #[error("{0}")]
    Protocol(#[from] ProtocolError),

    #[error("{0}")]
    ProtocolEncoding(#[from] EncodingError),

    #[error("Channel {channel_id} has unknown protocol version {protocol_version}")]
    UnknownProtocol {
        channel_id: String,
        protocol_version: String,
    },
}

impl From<FromUtf8Error> for ContractError {
    fn from(_: FromUtf8Error) -> Self {
        ContractError::Std(StdError::invalid_utf8("parsing denom key"))
    }
}