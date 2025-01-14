//! Utilities to make kitsune testing a little more sane.

#![allow(dead_code)]

use crate::types::actor::*;
use crate::types::agent_store::*;
use crate::types::event::*;
use crate::*;
use futures::future::FutureExt;
use ghost_actor::dependencies::tracing;
use std::collections::HashMap;
use std::sync::Arc;

/// Utility trait for test values
pub trait TestVal: Sized {
    /// Create the test val
    fn test_val() -> Self;
}

/// Boilerplate shortcut for implementing TestVal on an item
#[macro_export]
macro_rules! test_val  {
    ($($item:ty => $code:block,)*) => {$(
        impl TestVal for $item { fn test_val() -> Self { $code } }
    )*};
}

/// internal helper to generate randomized kitsune data items
fn rand36<F: KitsuneBinType>() -> Arc<F> {
    use rand::Rng;
    let mut out = vec![0; 36];
    rand::thread_rng().fill(&mut out[..]);
    Arc::new(F::new(out))
}

// setup randomized TestVal::test_val() impls for kitsune data items
test_val! {
    Arc<KitsuneSpace> => { rand36() },
    Arc<KitsuneAgent> => { rand36() },
    Arc<KitsuneBasis> => { rand36() },
    Arc<KitsuneOpHash> => { rand36() },
}

/// Create a handler task and produce a Sender for interacting with it
pub async fn spawn_handler<H: KitsuneP2pEventHandler + ghost_actor::GhostControlHandler>(
    h: H,
) -> (
    futures::channel::mpsc::Sender<event::KitsuneP2pEvent>,
    tokio::task::JoinHandle<ghost_actor::GhostResult<()>>,
) {
    let builder = ghost_actor::actor_builder::GhostActorBuilder::new();
    let (tx, rx) = futures::channel::mpsc::channel(4096);
    builder.channel_factory().attach_receiver(rx).await.unwrap();
    let driver = builder.spawn(h);
    (tx, tokio::task::spawn(driver))
}

pub fn hash_op_data(data: &[u8]) -> Arc<KitsuneOpHash> {
    Arc::new(KitsuneOpHash::new(
        blake2b_simd::Params::new()
            .hash_length(32)
            .hash(data)
            .as_bytes()
            .to_vec(),
    ))
}

/// Start a test signal server
pub async fn start_signal_srv() -> (std::net::SocketAddr, sbd_server::SbdServer) {
    let server = sbd_server::SbdServer::new(Arc::new(sbd_server::Config {
        bind: vec!["127.0.0.1:0".to_string(), "[::1]:0".to_string()],
        limit_clients: 100,
        ..Default::default()
    }))
    .await
    .unwrap();

    (*server.bind_addrs().first().unwrap(), server)
}

mod harness_event;
pub(crate) use harness_event::*;

mod harness_agent;
pub(crate) use harness_agent::*;

mod harness_actor;
#[allow(unused_imports)]
pub(crate) use harness_actor::*;

pub(crate) mod scenario_def_local;

pub mod data;
