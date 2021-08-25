//! ockam_node - Ockam Node API
#![deny(
    missing_docs,
    dead_code,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate tracing;

#[cfg(feature = "std")]
pub use tokio;
#[cfg(not(feature = "std"))]
pub use ockam_executor::{interrupt, tokio}; // TODO move interrupt to ockam_core

mod address_record;
mod context;
mod error;
mod executor;
mod mailbox;
mod messages;
mod node;
mod parser;
mod relay;
mod router;
mod tests;

pub(crate) use address_record::*;
pub use context::*;
pub use executor::*;
pub use mailbox::*;
pub use messages::*;

pub use node::{start_node, NullWorker};

#[cfg(feature = "std")]
use core::future::Future;
#[cfg(feature = "std")]
use tokio::{runtime::Runtime, task};

/// Execute a future without blocking the executor
///
/// This is a wrapper around two simple tokio functions to allow
/// ockam_node to wait for a task to be completed in a non-async
/// environment.
///
/// This function is not meant to be part of the ockam public API, but
/// as an implementation utility for other ockam utilities that use
/// tokio.
#[doc(hidden)]
#[cfg(feature = "std")]
pub fn block_future<'r, F>(rt: &'r Runtime, f: F) -> <F as Future>::Output
where
    F: Future + Send,
    F::Output: Send,
{
    println!("block_future::begin");

    //let result = spin_on::spin_on(f);
    let result = task::block_in_place(
        move || {
            let local = task::LocalSet::new();
            local.block_on(rt, f)
    });
    //let result = my_spawn(rt, f);

    println!("block_future::end");
    return result;
}
#[cfg(not(feature = "std"))]
pub use crate::tokio::runtime::block_future;

#[doc(hidden)]
#[cfg(feature = "std")]
pub fn spawn<F: 'static>(f: F)
where
    F: Future + Send,
    F::Output: Send,
{
    task::spawn(f);
}
