use crate::Context;
use ockam_core::{Address, Any, LocalMessage, Result, Route, Routed, TransportMessage, Worker};
use tracing::info;

/// Alias worker to register remote workers under local names.
///
/// To talk with this worker, you can use the [`RemoteForwarder`](crate::RemoteForwarder)
/// which is a compatible client for this server.
#[non_exhaustive]
pub struct ForwardingService;

impl ForwardingService {
    /// Start a forwarding service. The address of the forwarding service will be
    /// `"forwarding_service"`.
    pub async fn create(ctx: &Context) -> Result<()> {
        ctx.start_worker("forwarding_service", Self).await?;
        Ok(())
    }
}

#[crate::worker]
impl Worker for ForwardingService {
    type Context = Context;
    type Message = Any;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        let forward_route = msg.return_route();
        let payload = msg.into_transport_message().payload;
        Forwarder::create(ctx, forward_route, payload).await?;

        Ok(())
    }
}

struct Forwarder {
    forward_route: Route,
    // this option will be `None` after this worker is initialized, because
    // while initializing, the worker will send the payload contained in this
    // field to the `forward_route`, to indicate a successful connection
    payload: Option<Vec<u8>>,
}

impl Forwarder {
    async fn create(
        ctx: &Context,
        forward_route: Route,
        registration_payload: Vec<u8>,
    ) -> Result<()> {
        info!("Created new alias for {}", forward_route);
        let address = Address::random(0);
        let forwarder = Self {
            forward_route,
            payload: Some(registration_payload),
        };
        ctx.start_worker(address, forwarder).await?;

        Ok(())
    }
}

#[crate::worker]
impl Worker for Forwarder {
    type Context = Context;
    type Message = Any;

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        let payload = self
            .payload
            .take()
            .expect("payload must be available on init");
        let msg = TransportMessage::v1(self.forward_route.clone(), ctx.address(), payload);

        ctx.forward(LocalMessage::new(msg, Vec::new())).await?;

        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        info!(
            "Alias forward from {} to {}",
            msg.sender(),
            self.forward_route.next().unwrap(),
        );
        let mut msg = msg.into_transport_message();
        msg.onward_route = self.forward_route.clone();

        ctx.forward(LocalMessage::new(msg, Vec::new())).await?;

        Ok(())
    }
}
