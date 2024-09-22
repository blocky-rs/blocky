use crate::{decoder::Decoder, encoder::Encoder};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PacketFlow {
    Clientbound,
    Serverbound,
}

pub trait Packet: Encoder + Decoder {}
