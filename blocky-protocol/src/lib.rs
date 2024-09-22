pub mod handshake {
    use blocky_derive::Packet;
    use blocky_net::types::VarInt;

    // clientbound

    // serverbound

    #[derive(Packet)]
    pub struct Handshake {
        pub protocol_version: VarInt,
        pub server_address: String,
        pub server_port: u16,
        pub next_state: VarInt,
    }
}

pub mod status {
    use blocky_derive::Packet;

    // clientbound

    #[derive(Packet)]
    pub struct StatusResponse {
        pub status: String,
    }

    #[derive(Packet)]
    pub struct PongResponse {
        pub time: u64,
    }

    // serverbound

    #[derive(Packet)]
    pub struct StatusRequest;

    #[derive(Packet)]
    pub struct PingRequest {
        pub time: u64,
    }
}

pub mod login {
    use blocky_derive::{Decoder, Encoder, Packet};
    use blocky_net::types::{LengthInferredVecU8, LengthPrefixedVec, LengthPrefixedVecU8, VarInt};
    use uuid::Uuid;

    // clientbound

    #[derive(Packet)]
    pub struct Disconnect {
        pub reason: String, // TODO: This should be a JSON Text Component (need to add)
    }

    #[derive(Packet)]
    pub struct EncryptionRequest {
        pub server_id: String,
        pub public_key: LengthPrefixedVecU8<VarInt>,
        pub verify_token: LengthPrefixedVecU8<VarInt>,
        pub should_authenticate: bool,
    }

    #[derive(Encoder, Decoder)]
    pub struct LoginSuccessProperty {
        pub name: String,
        pub value: String,
        pub signature: Option<String>,
    }

    #[derive(Packet)]
    pub struct LoginSuccess {
        pub uuid: Uuid,
        pub username: String,
        pub properties: LengthPrefixedVec<VarInt, LoginSuccessProperty>,
        pub strict_error_handling: bool,
    }

    #[derive(Packet)]
    pub struct SetCompression {
        pub threshold: VarInt,
    }

    #[derive(Packet)]
    pub struct LoginPluginRequest {
        pub message_id: VarInt,
        pub channel: String, // TODO: Implement resource location / identifiers
        pub data: LengthInferredVecU8,
    }

    #[derive(Packet)]
    pub struct CookieRequest {
        pub key: String, // TODO: Implement resource location / identifiers
    }

    // serverbound

    #[derive(Packet)]
    pub struct LoginStart {
        pub name: String,
        pub uuid: Uuid,
    }

    #[derive(Packet)]
    pub struct EncryptionResponse {
        pub shared_secret: LengthPrefixedVec<VarInt, u8>,
        pub verify_token: LengthPrefixedVec<VarInt, u8>,
    }

    #[derive(Packet)]
    pub struct LoginPluginResponse {
        pub message_id: VarInt,
        pub success: bool,
        pub data: LengthInferredVecU8,
    }

    #[derive(Packet)]
    pub struct LoginAcknowledged;

    #[derive(Packet)]
    pub struct CookieResponse {
        pub key: String, // TODO: Implement resource location / identifiers
        pub payload: Option<LengthPrefixedVecU8<VarInt>>,
    }
}
