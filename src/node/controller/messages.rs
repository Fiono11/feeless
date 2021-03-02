use super::Controller;
use crate::node::cookie::Cookie;
use crate::node::header::{Extensions, Header, MessageType};
use crate::node::messages::confirm_ack::ConfirmAck;
use crate::node::messages::confirm_req::ConfirmReq;
use crate::node::messages::handshake::{Handshake, HandshakeQuery, HandshakeResponse};
use crate::node::messages::keepalive::Keepalive;
use crate::node::messages::publish::Publish;
use crate::node::messages::telemetry_ack::TelemetryAck;
use crate::node::messages::telemetry_req::TelemetryReq;
use crate::{Public, Seed, Signature};
use anyhow::Context;
use tracing::{debug, instrument, trace, warn};

impl Controller {
    #[instrument(skip(self))]
    pub async fn send_handshake(&mut self) -> anyhow::Result<()> {
        trace!("Sending handshake");
        self.send_header(MessageType::Handshake, *Extensions::new().query())
            .await?;

        // TODO: Track our own cookie?
        let cookie = Cookie::random();
        self.state
            .lock()
            .await
            .set_cookie(self.peer_addr, cookie.clone())
            .await?;
        let handshake_query = HandshakeQuery::new(cookie);
        self.send(&handshake_query).await?;

        Ok(())
    }

    pub async fn handle_handshake(
        &mut self,
        header: &Header,
        handshake: Handshake,
    ) -> anyhow::Result<()> {
        enum ShouldRespond {
            No,
            Yes(Public, Signature),
        }
        let mut should_respond = ShouldRespond::No;

        if header.ext().is_query() {
            // This would probably be a programming error if it panicked.
            let query = handshake.query.expect("query is None but is_query is True");

            // XXX: Hacky code here just to see if it works!
            // TODO: Move into state
            let seed = Seed::random();
            let private = seed.derive(0);
            let public = private.to_public();
            let signature = private.sign(query.cookie().as_bytes())?;
            public
                .verify(query.cookie().as_bytes(), &signature)
                .context("Recv node id handshake")?;

            // Respond at the end because we mess with the header buffer.
            should_respond = ShouldRespond::Yes(public, signature);
        }

        if header.ext().is_response() {
            let response = handshake
                .response
                .expect("response is None but is_response is True");
            let public = response.public;
            let signature = response.signature;

            // TODO: Move to controller
            let cookie = &self
                .state
                .lock()
                .await
                .cookie_for_socket_addr(&self.peer_addr)
                .await?;
            if cookie.is_none() {
                warn!(
                    "Peer {:?} has no cookie. Can't verify handshake.",
                    self.peer_addr
                );
                return Ok(());
            }
            let cookie = cookie.as_ref().unwrap();

            public
                .verify(&cookie.as_bytes(), &signature)
                .context("Invalid signature in node_id_handshake response")?;
        }

        if let ShouldRespond::Yes(public, signature) = should_respond {
            let mut header = self.header;
            header.reset(MessageType::Handshake, *Extensions::new().response());
            self.send(&header).await?;

            let response = HandshakeResponse::new(public, signature);
            self.send(&response).await?;
        }

        Ok(())
    }

    pub async fn handle_keepalive(
        &mut self,
        header: &Header,
        keepalive: Keepalive,
    ) -> anyhow::Result<()> {
        dbg!(keepalive);
        Ok(())
    }

    pub async fn handle_telemetry_req(
        &mut self,
        header: &Header,
        telemetry_req: TelemetryReq,
    ) -> anyhow::Result<()> {
        dbg!(telemetry_req);
        Ok(())
    }

    pub async fn handle_telemetry_ack(
        &mut self,
        header: &Header,
        telemetry_ack: TelemetryAck,
    ) -> anyhow::Result<()> {
        dbg!(telemetry_ack);
        Ok(())
    }

    pub async fn handle_publish(
        &mut self,
        header: &Header,
        publish: Publish,
    ) -> anyhow::Result<()> {
        dbg!(publish);
        Ok(())
    }

    pub async fn handle_confirm_req(
        &mut self,
        header: &Header,
        confirm_req: ConfirmReq,
    ) -> anyhow::Result<()> {
        dbg!(confirm_req);
        Ok(())
    }

    pub async fn handle_confirm_ack(
        &mut self,
        header: &Header,
        confirm_ack: ConfirmAck,
    ) -> anyhow::Result<()> {
        dbg!(confirm_ack);
        Ok(())
    }
}