use std::{fmt::Display, marker::PhantomData};

use chain_utils::GetChain;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use macros::apply;
use queue_msg::{aggregate, fetch, wait, HandleEvent, QueueError, QueueMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use unionlabs::{
    hash::H256,
    proof::{ChannelEndPath, ClientStatePath, ConnectionPath},
    traits::{ClientIdOf, ClientTypeOf, HeightOf},
};

use crate::{
    aggregate::{
        mk_aggregate_wait_for_update, Aggregate, AggregateChannelHandshakeUpdateClient,
        AggregateConnectionFetchFromChannelEnd, AggregateConnectionOpenAck,
        AggregateConnectionOpenConfirm, AggregateConnectionOpenTry, AggregateMsgAfterUpdate,
        AggregatePacketUpdateClient, AggregateUpdateClientFromClientId, AnyAggregate,
        ChannelHandshakeEvent, PacketEvent,
    },
    any_enum, any_lc,
    fetch::{AnyFetch, Fetch, FetchLatestClientState, FetchState},
    id, identified, seq,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, ChainExt, RelayerMsgTypes,
};

#[apply(any_enum)]
#[any = AnyEvent]
pub enum Event<Hc: ChainExt, Tr: ChainExt> {
    Ibc(IbcEvent<Hc, Tr>),
    Command(Command<Hc, Tr>),
}

impl HandleEvent<RelayerMsgTypes> for AnyLightClientIdentified<AnyEvent> {
    fn handle(
        self,
        store: &<RelayerMsgTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<RelayerMsgTypes>, QueueError> {
        let wait = self;

        any_lc! {
            |wait| {
                store
                    .with_chain(&wait.chain_id, move |c| wait.t.handle(c))
                    .map_err(|e| QueueError::Fatal(Box::new(e)))
            }
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Event<Hc, Tr> {
    pub fn handle(self, hc: Hc) -> QueueMsg<RelayerMsgTypes>
    where
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
    {
        match self {
            Event::Ibc(ibc_event) => match ibc_event.event {
                unionlabs::events::IbcEvent::CreateClient(e) => {
                    tracing::info!("client created: {e:?}");

                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::UpdateClient(e) => {
                    tracing::info!(
                        "client updated: {:?} to {:?}",
                        e.client_id,
                        e.consensus_heights
                    );

                    QueueMsg::Noop
                }

                unionlabs::events::IbcEvent::ClientMisbehaviour(_) => unimplemented!(),
                unionlabs::events::IbcEvent::SubmitEvidence(_) => unimplemented!(),

                unionlabs::events::IbcEvent::ConnectionOpenInit(init) => seq([
                    wait(id(
                        hc.chain_id(),
                        WaitForBlock {
                            height: ibc_event.height,
                            __marker: PhantomData,
                        },
                    )),
                    aggregate(
                        [mk_aggregate_wait_for_update(
                            hc.chain_id(),
                            init.client_id.clone(),
                            init.counterparty_client_id.clone(),
                            ibc_event.height,
                        )],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateMsgAfterUpdate::ConnectionOpenTry(
                                AggregateConnectionOpenTry {
                                    event_height: ibc_event.height,
                                    event: init,
                                },
                            ),
                        ),
                    ),
                ]),
                unionlabs::events::IbcEvent::ConnectionOpenTry(try_) => seq([aggregate(
                    [mk_aggregate_wait_for_update(
                        hc.chain_id(),
                        try_.client_id.clone(),
                        try_.counterparty_client_id.clone(),
                        ibc_event.height,
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateMsgAfterUpdate::ConnectionOpenAck(AggregateConnectionOpenAck {
                            event_height: ibc_event.height,
                            event: try_,
                        }),
                    ),
                )]),
                unionlabs::events::IbcEvent::ConnectionOpenAck(ack) => aggregate(
                    [mk_aggregate_wait_for_update(
                        hc.chain_id(),
                        ack.client_id.clone(),
                        ack.counterparty_client_id.clone(),
                        ibc_event.height,
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateMsgAfterUpdate::ConnectionOpenConfirm(
                            AggregateConnectionOpenConfirm {
                                event_height: ibc_event.height,
                                event: ack,
                            },
                        ),
                    ),
                ),
                unionlabs::events::IbcEvent::ConnectionOpenConfirm(confirm) => {
                    tracing::info!("connection opened: {confirm:?}");

                    QueueMsg::Noop
                }

                unionlabs::events::IbcEvent::ChannelOpenInit(init) => aggregate(
                    [aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ChannelEndPath {
                                    port_id: init.port_id.clone(),
                                    channel_id: init.channel_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateConnectionFetchFromChannelEnd {
                                at: ibc_event.height,
                                __marker: PhantomData,
                            },
                        ),
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateChannelHandshakeUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            channel_handshake_event: ChannelHandshakeEvent::Init(init),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::ChannelOpenTry(try_) => aggregate(
                    [aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ChannelEndPath {
                                    port_id: try_.port_id.clone(),
                                    channel_id: try_.channel_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            Aggregate::ConnectionFetchFromChannelEnd(
                                AggregateConnectionFetchFromChannelEnd {
                                    at: ibc_event.height,
                                    __marker: PhantomData,
                                },
                            ),
                        ),
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateChannelHandshakeUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            channel_handshake_event: ChannelHandshakeEvent::Try(try_),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::ChannelOpenAck(ack) => aggregate(
                    [aggregate(
                        [fetch(id::<Hc, Tr, _>(
                            hc.chain_id(),
                            FetchState {
                                at: ibc_event.height,
                                path: ChannelEndPath {
                                    port_id: ack.port_id.clone(),
                                    channel_id: ack.channel_id.clone(),
                                }
                                .into(),
                            },
                        ))],
                        [],
                        id(
                            hc.chain_id(),
                            AggregateConnectionFetchFromChannelEnd {
                                at: ibc_event.height,
                                __marker: PhantomData,
                            },
                        ),
                    )],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateChannelHandshakeUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            channel_handshake_event: ChannelHandshakeEvent::Ack(ack),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::ChannelOpenConfirm(confirm) => {
                    tracing::info!("channel opened: {confirm:?}");

                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::RecvPacket(packet) => aggregate(
                    [fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        FetchState {
                            at: ibc_event.height,
                            path: ConnectionPath {
                                connection_id: packet.connection_id.clone(),
                            }
                            .into(),
                        },
                    ))],
                    [],
                    id(
                        hc.chain_id(),
                        AggregatePacketUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            tx_hash: ibc_event.tx_hash,
                            packet_event: PacketEvent::Recv(packet),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::SendPacket(packet) => aggregate(
                    [fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        FetchState {
                            at: ibc_event.height,
                            path: ConnectionPath {
                                connection_id: packet.connection_id.clone(),
                            }
                            .into(),
                        },
                    ))],
                    [],
                    id(
                        hc.chain_id(),
                        AggregatePacketUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            tx_hash: ibc_event.tx_hash,
                            packet_event: PacketEvent::Send(packet),
                            __marker: PhantomData,
                        },
                    ),
                ),
                unionlabs::events::IbcEvent::AcknowledgePacket(ack) => {
                    tracing::info!(?ack, "packet acknowledged");
                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::TimeoutPacket(timeout) => {
                    tracing::error!(?timeout, "packet timed out");
                    QueueMsg::Noop
                }
                unionlabs::events::IbcEvent::WriteAcknowledgement(write_ack) => {
                    tracing::info!(?write_ack, "packet acknowledgement written");
                    QueueMsg::Noop
                }
            },
            Event::Command(command) => match command {
                Command::UpdateClient {
                    client_id,
                    counterparty_client_id,
                } => aggregate(
                    [fetch(crate::id::<Hc, Tr, _>(
                        hc.chain_id(),
                        FetchLatestClientState {
                            path: ClientStatePath {
                                client_id: client_id.clone(),
                            },
                            __marker: PhantomData,
                        },
                    ))],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateUpdateClientFromClientId {
                            client_id,
                            counterparty_client_id,
                        },
                    ),
                ),
            },
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Display for Event<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Ibc(_) => write!(f, "Ibc"),
            Event::Command(cmd) => write!(f, "{cmd}"),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub struct IbcEvent<Hc: ChainExt, Tr: ChainExt> {
    pub tx_hash: H256,
    pub height: HeightOf<Hc>,
    pub event: unionlabs::events::IbcEvent<ClientIdOf<Hc>, ClientTypeOf<Hc>, ClientIdOf<Tr>>,
}

impl<Hc: ChainExt, Tr: ChainExt> Display for IbcEvent<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.event.name())
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(
    bound(serialize = "", deserialize = ""),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case"
)]
#[display(fmt = "Command::{}")]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
)]
pub enum Command<Hc: ChainExt, Tr: ChainExt> {
    #[display(fmt = "UpdateClient({client_id}, {counterparty_client_id})")]
    UpdateClient {
        client_id: ClientIdOf<Hc>,
        // TODO: This should only take a counterparty chain id, since light clients track chains, not each other, and it should be possible to update a client without it having a counterparty.
        counterparty_client_id: ClientIdOf<Tr>,
    },
}
