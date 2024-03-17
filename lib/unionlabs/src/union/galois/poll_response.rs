use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    union::galois::prove_response::{ProveResponse, TryFromProveResponseError},
};

#[model(proto(raw(protos::union::galois::api::v2::PollResponse), into, from))]
pub enum PollResponse {
    Pending,
    Failed(ProveRequestFailed),
    Done(ProveRequestDone),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveRequestFailed {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveRequestDone {
    pub response: ProveResponse,
}

impl From<PollResponse> for protos::union::galois::api::v2::PollResponse {
    fn from(value: PollResponse) -> Self {
        Self {
            result: Some(match value {
                PollResponse::Pending => {
                    protos::union::galois::api::v2::poll_response::Result::Pending(
                        protos::union::galois::api::v2::ProveRequestPending {},
                    )
                }
                PollResponse::Failed(failed) => {
                    protos::union::galois::api::v2::poll_response::Result::Failed(
                        protos::union::galois::api::v2::ProveRequestFailed {
                            message: failed.message,
                        },
                    )
                }
                PollResponse::Done(done) => {
                    protos::union::galois::api::v2::poll_response::Result::Done(
                        protos::union::galois::api::v2::ProveRequestDone {
                            response: Some(done.response.into()),
                        },
                    )
                }
            }),
        }
    }
}

#[derive(Debug)]
pub enum TryFromPollResponseError {
    MissingField(MissingField),
    ProveResponse(TryFromProveResponseError),
}

impl TryFrom<protos::union::galois::api::v2::PollResponse> for PollResponse {
    type Error = TryFromPollResponseError;

    fn try_from(value: protos::union::galois::api::v2::PollResponse) -> Result<Self, Self::Error> {
        match required!(value.result)? {
            protos::union::galois::api::v2::poll_response::Result::Pending(_) => Ok(Self::Pending),
            protos::union::galois::api::v2::poll_response::Result::Failed(failed) => {
                Ok(Self::Failed(ProveRequestFailed {
                    message: failed.message,
                }))
            }
            protos::union::galois::api::v2::poll_response::Result::Done(done) => {
                Ok(Self::Done(ProveRequestDone {
                    response: required!(done.response)?
                        .try_into()
                        .map_err(TryFromPollResponseError::ProveResponse)?,
                }))
            }
        }
    }
}
