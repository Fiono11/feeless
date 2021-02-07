use crate::cookie::Cookie;
use crate::header::{BlockType, Extensions, Header, MessageType};
use crate::messages::confirm_ack::ConfirmAck;
use crate::messages::confirm_req::ConfirmReq;
use crate::messages::node_id_handshake::{NodeIdHandshakeQuery, NodeIdHandshakeResponse};
use crate::messages::telemetry_req::TelemetryReq;
use crate::peer::Peer;
use crate::state::State;
use crate::state::{BoxedState, SledState};
use crate::wire::Wire;

use crate::messages::publish::Publish;
use anyhow::anyhow;
use feeless::{expect_len, to_hex, Seed};
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, instrument, trace, warn};

/// A connection to a single peer.
#[derive(Debug)]
pub struct Channel {
    pub state: BoxedState,

    // TODO: Both of these into a Communication trait, for ease of testing. e.g.:
    //  * async fn Comm::send() -> Result<()>
    //  * async fn Comm::recv() -> Result<()>
    //  * fn Comm::address() -> String
    //
    // This would also remove Self::buffer.
    // Not sure about the performance problems of having to use async-trait.
    stream: TcpStream,
    pub(crate) peer_addr: SocketAddr,

    /// A reusable header to reduce allocations.
    pub(crate) header: Header,

    /// Storage that can be shared within this task without reallocating.
    /// This is currently only used for the recv buffers.
    buffer: Vec<u8>,
}

impl Channel {
    pub fn new(state: BoxedState, stream: TcpStream) -> Self {
        let network = state.network();
        // TODO: Remove unwrap
        let peer_addr = stream.peer_addr().unwrap();
        Self {
            state,
            stream,
            peer_addr,
            header: Header::new(network, MessageType::NodeIdHandshake, Extensions::new()),
            buffer: Vec::with_capacity(1024),
        }
    }

    #[instrument(skip(self, header))]
    async fn recv<T: Wire + Debug>(&mut self, header: Option<&Header>) -> anyhow::Result<T> {
        let expected_len = T::len(header)?;
        if expected_len > self.buffer.len() {
            trace!("Expanding buffer {} -> {}", self.buffer.len(), expected_len);
            self.buffer.resize(expected_len, 0)
        }

        let buffer = &mut self.buffer[0..expected_len];
        let bytes_read = self.stream.read_exact(buffer).await?;
        expect_len(bytes_read, expected_len, "Recv packet")?;
        trace!("HEX: {}", to_hex(&buffer));

        let buffer = &self.buffer[0..expected_len];
        let result = T::deserialize(header, buffer)?;
        debug!("OBJ: {:?}", &result);

        Ok(result)
    }

    async fn todo_dump(&mut self) -> anyhow::Result<()> {
        loop {
            let mut c = [0u8];
            self.stream.read(&mut c).await?;
            print!("{}", to_hex(&c));
        }
        todo!();
    }

    #[instrument(level = "debug", skip(self, message))]
    async fn send<T: Wire + Debug>(&mut self, message: &T) -> anyhow::Result<()> {
        let data = message.serialize();
        trace!("HEX {}", to_hex(&data));
        debug!("OBJ {:?}", &message);
        self.stream.write_all(&data).await?;
        Ok(())
    }

    async fn send_header(
        &mut self,
        message_type: MessageType,
        ext: Extensions,
    ) -> anyhow::Result<()> {
        let mut header = self.header;
        header.reset(message_type, ext);
        Ok(self.send(&header).await?)
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.send_node_id_handshake().await?;
        self.send_telemetry_req().await?;

        loop {
            let header = self.recv::<Header>(None).await?;
            header.validate(&self.state)?;
            // debug!("Header: {:?}", &header);

            match header.message_type() {
                MessageType::Keepalive => self.recv_keepalive(header).await?,
                MessageType::Publish => self.recv_publish(header).await?,
                MessageType::ConfirmReq => self.recv_confirm_req(header).await?,
                MessageType::ConfirmAck => self.recv_confirm_ack(header).await?,
                // MessageType::BulkPull => todo!(),
                // MessageType::BulkPush => todo!(),
                // MessageType::FrontierReq => todo!(),
                MessageType::NodeIdHandshake => self.recv_node_id_handshake(header).await?,
                // MessageType::BulkPullAccount => todo!(),
                MessageType::TelemetryReq => self.recv_telemetry_req(header).await?,
                // MessageType::TelemetryAck => todo!(),
                _ => todo!("{:?}", header),
            }
        }
    }

    #[instrument(skip(self, header))]
    async fn recv_keepalive(&mut self, header: Header) -> anyhow::Result<()> {
        for _ in 0..8 {
            let peer = self.recv::<Peer>(Some(&header)).await?;
            debug!("Keepalive peer: {:?}", peer);
        }
        Ok(())
    }

    #[instrument(skip(self, header))]
    async fn recv_publish(&mut self, header: Header) -> anyhow::Result<()> {
        self.recv::<Publish>(Some(&header)).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn send_node_id_handshake(&mut self) -> anyhow::Result<()> {
        self.send_header(MessageType::NodeIdHandshake, *Extensions::new().query())
            .await?;

        let cookie = Cookie::random();
        self.state
            .set_cookie(self.peer_addr, cookie.clone())
            .await?;
        let handshake_query = NodeIdHandshakeQuery::new(cookie);
        self.send(&handshake_query).await?;

        Ok(())
    }

    #[instrument(skip(self, header))]
    async fn recv_node_id_handshake(&mut self, header: Header) -> anyhow::Result<()> {
        if header.ext().is_query() {
            let query = self.recv::<NodeIdHandshakeQuery>(Some(&header)).await?;
            // XXX: Hacky code here just to see if it works!
            // TODO: Move into state
            let seed = Seed::random();
            let private = seed.derive(0);
            let public = private.to_public();
            let signature = private.sign(query.cookie().as_bytes())?;
            debug_assert!(public.verify(query.cookie().as_bytes(), &signature));

            let mut header = self.header;
            header.reset(MessageType::NodeIdHandshake, *Extensions::new().response());
            self.send(&header).await?;

            let response = NodeIdHandshakeResponse::new(public, signature);
            self.send(&response).await?;
        }

        if header.ext().is_response() {
            let response = self.recv::<NodeIdHandshakeResponse>(Some(&header)).await?;
            let public = response.public;
            let signature = response.signature;

            let cookie = &self.state.cookie_for_socket_addr(&self.peer_addr).await?;
            if cookie.is_none() {
                warn!(
                    "Peer {:?} has no cookie. Can't verify handshake.",
                    self.peer_addr
                );
                return Ok(());
            }
            let cookie = cookie.as_ref().unwrap();

            if !public.verify(&cookie.as_bytes(), &signature) {
                return Err(anyhow!("Invalid signature in node_id_handshake response"));
            }
        }

        Ok(())
    }

    #[instrument(skip(self, header))]
    async fn recv_confirm_req(&mut self, header: Header) -> anyhow::Result<()> {
        let data = self.recv::<ConfirmReq>(Some(&header)).await?;
        trace!("Pairs: {:?}", &data);
        warn!("TODO confirm_req");
        Ok(())
    }

    #[instrument(skip(self, header))]
    async fn recv_confirm_ack(&mut self, header: Header) -> anyhow::Result<()> {
        let data = self.recv::<ConfirmAck>(Some(&header)).await?;
        warn!("TODO confirm_ack");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn recv_telemetry_req(&mut self, header: Header) -> anyhow::Result<()> {
        self.recv::<TelemetryReq>(Some(&header)).await?;
        warn!("TODO telemetry_req");
        Ok(())
    }
    #[instrument(skip(self))]
    async fn send_telemetry_req(&mut self) -> anyhow::Result<()> {
        self.send_header(MessageType::TelemetryReq, Extensions::new())
            .await?;
        Ok(())
    }
}