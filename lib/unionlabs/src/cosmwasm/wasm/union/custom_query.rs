use std::fmt::Debug;

use cosmwasm_std::{to_json_vec, Binary, ContractResult, Deps, Env, QueryRequest, SystemResult};
use prost::Message;
use protos::cosmos::base::tendermint::v1beta1::AbciQueryResponse;

use crate::{
    bls::BlsPublicKey,
    google::protobuf::any::Any,
    ibc::core::client::height::Height,
    proof::{ClientConsensusStatePath, Path},
    TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("error while running `fast_aggregate_verify` query ({0})")]
    FastAggregateVerify(String),
    #[error("error while running `aggregate_public_keys` query ({0})")]
    AggregatePublicKeys(String),
    #[error("invalid public key is returned from `aggregate_public_key`")]
    InvalidAggregatePublicKey,
    #[error("error while running `consensus_state` query ({0})")]
    ConsensusState(String),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UnionCustomQuery {
    AggregateVerify {
        public_keys: Vec<Binary>,
        message: Binary,
        signature: Binary,
    },
    Aggregate {
        public_keys: Vec<Binary>,
    },
    ConsensusState {
        client_id: String,
        height: Height,
    },
    ClientState {
        client_id: String,
    },
}

impl cosmwasm_std::CustomQuery for UnionCustomQuery {}

pub fn query_fast_aggregate_verify(
    deps: Deps<UnionCustomQuery>,
    public_keys: Vec<Binary>,
    message: Binary,
    signature: Binary,
) -> Result<bool, Error> {
    let request: QueryRequest<UnionCustomQuery> =
        QueryRequest::Custom(UnionCustomQuery::AggregateVerify {
            public_keys,
            message,
            signature,
        });
    deps.querier
        .query(&request)
        .map_err(|e| Error::FastAggregateVerify(e.to_string()))
}

pub fn query_aggregate_public_keys(
    deps: Deps<UnionCustomQuery>,
    public_keys: Vec<BlsPublicKey>,
) -> Result<BlsPublicKey, Error> {
    let request: QueryRequest<UnionCustomQuery> =
        QueryRequest::Custom(UnionCustomQuery::Aggregate {
            public_keys: public_keys.into_iter().map(|x| Binary(x.into())).collect(),
        });
    let response: Binary = deps
        .querier
        .query(&request)
        .map_err(|e| Error::AggregatePublicKeys(e.to_string()))?;
    response
        .0
        .as_slice()
        .try_into()
        .map_err(|_| Error::InvalidAggregatePublicKey)
}

#[allow(clippy::missing_panics_doc)]
pub fn query_consensus_state<T>(
    deps: Deps<UnionCustomQuery>,
    env: &Env,
    client_id: String,
    height: Height,
) -> Result<T, Error>
where
    Any<T>: TryFromProto,
    TryFromProtoBytesError<TryFromProtoErrorOf<Any<T>>>: Debug,
{
    let query = protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
        data: Path::ClientConsensusStatePath(ClientConsensusStatePath { client_id, height })
            .to_string()
            .into_bytes(),
        path: "store/ibc/key".to_string(),
        height: env
            .block
            .height
            .wrapping_sub(1)
            .try_into()
            .expect("impossible"),
        prove: false,
    };
    let raw = to_json_vec(&QueryRequest::<UnionCustomQuery>::Stargate {
        path: "/cosmos.base.tendermint.v1beta1.Service/ABCIQuery".into(),
        data: query.encode_to_vec().into(),
    })
    .map_err(|e| Error::ConsensusState(format!("{e:?}")))?;
    let abci_response_data = match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(Error::ConsensusState(format!(
            "Querier system error: {system_err}"
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(Error::ConsensusState(format!(
            "Querier contract error: {contract_err}"
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }?;
    let abci_response = AbciQueryResponse::decode(abci_response_data.as_ref())
        .map_err(|e| Error::ConsensusState(format!("{e:?}")))?;
    let Any(value) = Any::<T>::try_from_proto_bytes(&abci_response.value)
        .map_err(|e| Error::ConsensusState(format!("{e:?}")))?;
    Ok(value)
}
