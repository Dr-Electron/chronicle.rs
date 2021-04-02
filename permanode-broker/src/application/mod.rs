// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{
    archiver::*,
    collector::*,
    listener::*,
    mqtt::*,
    solidifier::*,
    syncer::*,
    websocket::*,
};
use async_trait::async_trait;
pub(crate) use bee_common::packable::Packable;
pub(crate) use bee_message::{
    Message,
    MessageId,
};
pub use chronicle::*;
pub use log::*;
pub(crate) use paho_mqtt::{
    AsyncClient,
    CreateOptionsBuilder,
};
use permanode_common::{
    config::{
        BrokerConfig,
        StorageConfig,
    },
    SyncRange,
};
pub(crate) use permanode_storage::access::*;
use serde::{
    Deserialize,
    Serialize,
};
use std::ops::Range;
pub(crate) use std::{
    collections::HashMap,
    convert::TryFrom,
    net::SocketAddr,
    ops::{
        Deref,
        DerefMut,
    },
    path::PathBuf,
};
pub use tokio::{
    spawn,
    sync::mpsc,
};

mod event_loop;
mod init;
mod starter;
mod terminating;

/// Define the application scope trait
pub trait PermanodeBrokerScope: LauncherSender<PermanodeBrokerBuilder<Self>> {}
impl<H: LauncherSender<PermanodeBrokerBuilder<H>>> PermanodeBrokerScope for H {}

// Scylla builder
builder!(
    #[derive(Clone)]
    PermanodeBrokerBuilder<H> {
        listen_address: SocketAddr,
        listener_handle: ListenerHandle,
        logs_dir_path: PathBuf,
        collectors_count: u8,
        broker_config: BrokerConfig,
        storage_config: StorageConfig
});

#[derive(Deserialize, Serialize)]
/// It's the Interface of the broker app to dynamiclly configure the application during runtime
pub enum PermanodeBrokerThrough {
    /// Shutdown json to gracefully shutdown broker app
    Shutdown,
    Topology(Topology),
    ExitProgram,
}

/// BrokerHandle to be passed to the children
pub struct BrokerHandle<H: PermanodeBrokerScope> {
    tx: tokio::sync::mpsc::UnboundedSender<BrokerEvent<H::AppsEvents>>,
}
/// BrokerInbox used to recv events
pub struct BrokerInbox<H: PermanodeBrokerScope> {
    rx: tokio::sync::mpsc::UnboundedReceiver<BrokerEvent<H::AppsEvents>>,
}

impl<H: PermanodeBrokerScope> Clone for BrokerHandle<H> {
    fn clone(&self) -> Self {
        BrokerHandle::<H> { tx: self.tx.clone() }
    }
}

/// Application state
pub struct PermanodeBroker<H: PermanodeBrokerScope> {
    service: Service,
    websockets: HashMap<String, WsTx>,
    listener_handle: Option<ListenerHandle>,
    mqtt_handles: HashMap<String, MqttHandle>,
    asked_to_shutdown: HashMap<String, ()>,
    collectors_count: u8,
    collector_handles: HashMap<u8, CollectorHandle>,
    solidifier_handles: HashMap<u8, SolidifierHandle>,
    logs_dir_path: PathBuf,
    handle: Option<BrokerHandle<H>>,
    inbox: BrokerInbox<H>,
    default_keyspace: PermanodeKeyspace,
    sync_range: SyncRange,
    sync_data: SyncData,
    syncer_handle: Option<SyncerHandle>,
    broker_config: BrokerConfig,
    storage_config: Option<StorageConfig>,
}

/// SubEvent type, indicated the children
pub enum BrokerChild {
    /// Used by Listener to keep broker up to date with its service
    Listener(Service),
    /// Used by Mqtt to keep Broker up to date with its service
    Mqtt(Service, Option<MqttHandle>, Result<(), Need>),
    /// Used by Collector(s) to keep Broker up to date with its service
    Collector(Service),
    /// Used by Solidifier(s) to keep Broker up to date with its service
    Solidifier(Service),
    /// Used by Archiver to keep Broker up to date with its service
    Archiver(Service, Result<(), Need>),
    /// Used by Websocket to keep Broker up to date with its service
    Websocket(Service, Option<WsTx>),
}

/// Event type of the broker Application
pub enum BrokerEvent<T> {
    /// It's the passthrough event, which the scylla application will receive from
    Passthrough(T),
    /// Used by broker children to push their service
    Children(BrokerChild),
}

#[derive(Deserialize, Serialize, Debug)]
/// Topology event
pub enum Topology {
    AddMqttMessages(Url),
    AddMqttMessagesReferenced(Url),
    RemoveMqttMessages(Url),
    RemoveMqttMessagesReferenced(Url),
}

#[derive(Deserialize, Serialize)]
// use PermanodeBroker to indicate to the msg is from/to PermanodeBroker
pub enum SocketMsg<T> {
    PermanodeBroker(T),
}
#[derive(Debug, Clone)]
pub struct SyncData {
    /// The completed(synced and logged) milestones data
    pub(crate) completed: Vec<Range<u32>>,
    /// Synced milestones data but unlogged
    pub(crate) synced_but_unlogged: Vec<Range<u32>>,
    /// Gaps/missings milestones data
    pub(crate) gaps: Vec<Range<u32>>,
}

impl SyncData {
    pub fn take_lowest_gap(&mut self) -> Option<Range<u32>> {
        self.gaps.pop()
    }
    pub fn take_lowest_unlogged(&mut self) -> Option<Range<u32>> {
        self.synced_but_unlogged.pop()
    }
    pub fn take_lowest_gap_or_unlogged(&mut self) -> Option<Range<u32>> {
        let lowest_gap = self.gaps.last();
        let lowest_unlogged = self.synced_but_unlogged.last();
        match (lowest_gap, lowest_unlogged) {
            (Some(gap), Some(unlogged)) => {
                if gap.start < unlogged.start {
                    self.gaps.pop()
                } else {
                    self.synced_but_unlogged.pop()
                }
            }
            (Some(_), None) => self.gaps.pop(),
            (None, Some(_)) => self.synced_but_unlogged.pop(),
            _ => None,
        }
    }
    pub fn take_lowest_uncomplete(&mut self) -> Option<Range<u32>> {
        if let Some(mut pre_range) = self.take_lowest_gap_or_unlogged() {
            loop {
                if let Some(next_range) = self.get_lowest_gap_or_unlogged() {
                    if next_range.start.eq(&pre_range.end) {
                        pre_range.end = next_range.end;
                        let _ = self.take_lowest_gap_or_unlogged();
                    } else {
                        return Some(pre_range);
                    }
                } else {
                    return Some(pre_range);
                }
            }
        } else {
            None
        }
    }
    fn get_lowest_gap_or_unlogged(&self) -> Option<&Range<u32>> {
        let lowest_gap = self.gaps.last();
        let lowest_unlogged = self.synced_but_unlogged.last();
        match (lowest_gap, lowest_unlogged) {
            (Some(gap), Some(unlogged)) => {
                if gap.start < unlogged.start {
                    self.gaps.last()
                } else {
                    self.synced_but_unlogged.last()
                }
            }
            (Some(_), None) => self.gaps.last(),
            (None, Some(_)) => self.synced_but_unlogged.last(),
            _ => None,
        }
    }
}
/// implementation of the AppBuilder
impl<H: PermanodeBrokerScope> AppBuilder<H> for PermanodeBrokerBuilder<H> {}

/// implementation of through type
impl<H: PermanodeBrokerScope> ThroughType for PermanodeBrokerBuilder<H> {
    type Through = PermanodeBrokerThrough;
}

/// implementation of builder
impl<H: PermanodeBrokerScope> Builder for PermanodeBrokerBuilder<H> {
    type State = PermanodeBroker<H>;
    fn build(self) -> Self::State {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = Some(BrokerHandle { tx });
        let inbox = BrokerInbox { rx };
        let default_keyspace = PermanodeKeyspace::new(
            self.storage_config
                .as_ref()
                .and_then(|config| {
                    config
                        .keyspaces
                        .first()
                        .and_then(|keyspace| Some(keyspace.name.clone()))
                })
                .unwrap_or("permanode".to_owned()),
        );
        let sync_range = self
            .broker_config
            .as_ref()
            .and_then(|config| config.sync_range.and_then(|range| Some(range)))
            .unwrap_or(SyncRange::default());
        let sync_data = SyncData {
            completed: Vec::new(),
            synced_but_unlogged: Vec::new(),
            gaps: Vec::new(),
        };
        PermanodeBroker::<H> {
            service: Service::new(),
            websockets: HashMap::new(),
            listener_handle: self.listener_handle,
            mqtt_handles: HashMap::new(),
            asked_to_shutdown: HashMap::new(),
            collectors_count: self.collectors_count.unwrap_or(10),
            collector_handles: HashMap::new(),
            solidifier_handles: HashMap::new(),
            syncer_handle: None,
            logs_dir_path: self.logs_dir_path.expect("Expected logs directory path"),
            handle,
            inbox,
            default_keyspace,
            sync_range,
            sync_data,
            broker_config: self.broker_config.unwrap(),
            storage_config: self.storage_config,
        }
        .set_name()
    }
}

// TODO integrate well with other services;
/// implementation of passthrough functionality
impl<H: PermanodeBrokerScope> Passthrough<PermanodeBrokerThrough> for BrokerHandle<H> {
    fn launcher_status_change(&mut self, _service: &Service) {}
    fn app_status_change(&mut self, _service: &Service) {}
    fn passthrough(&mut self, _event: PermanodeBrokerThrough, _from_app_name: String) {}
    fn service(&mut self, _service: &Service) {}
}

/// implementation of shutdown functionality
impl<H: PermanodeBrokerScope> Shutdown for BrokerHandle<H> {
    fn shutdown(self) -> Option<Self>
    where
        Self: Sized,
    {
        let broker_shutdown: H::AppsEvents = serde_json::from_str("{\"PermanodeBroker\": \"Shutdown\"}").unwrap();
        let _ = self.send(BrokerEvent::Passthrough(broker_shutdown));
        None
    }
}

impl<H: PermanodeBrokerScope> Deref for BrokerHandle<H> {
    type Target = tokio::sync::mpsc::UnboundedSender<BrokerEvent<H::AppsEvents>>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl<H: PermanodeBrokerScope> DerefMut for BrokerHandle<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}

impl<H: PermanodeBrokerScope> Deref for BrokerInbox<H> {
    type Target = tokio::sync::mpsc::UnboundedReceiver<BrokerEvent<H::AppsEvents>>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<H: PermanodeBrokerScope> DerefMut for BrokerInbox<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

/// impl name of the application
impl<H: PermanodeBrokerScope> Name for PermanodeBroker<H> {
    fn set_name(mut self) -> Self {
        self.service.update_name("PermanodeBroker".to_string());
        self
    }
    fn get_name(&self) -> String {
        self.service.get_name()
    }
}
