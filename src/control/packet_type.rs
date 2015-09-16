use std::error::Error;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PacketType {
    /// Client request to connect to Server
    Connect(bool, bool, bool, bool),

    /// Connect acknowledgment
    ConnectAcknowledgement(bool, bool, bool, bool),

    /// Publish message
    Publish(bool, bool, bool, bool),

    /// Publish acknowledgment
    PublishAcknowledgement(bool, bool, bool, bool),

    /// Publish received (assured delivery part 1)
    PublishReceived(bool, bool, bool, bool),

    /// Publish release (assured delivery part 2)
    PublishRelease(bool, bool, bool, bool),

    /// Publish complete (assured delivery part 3)
    PublishComplete(bool, bool, bool, bool),

    /// Client subscribe request
    Subscribe(bool, bool, bool, bool),

    /// Subscribe acknowledgment
    SubscribeAcknowledgement(bool, bool, bool, bool),

    /// Unsubscribe request
    Unsubscribe(bool, bool, bool, bool),

    /// Unsubscribe acknowledgment
    UnsubscribeAcknowledgement(bool, bool, bool, bool),

    /// PING request
    PingRequest(bool, bool, bool, bool),

    /// PING response
    PingResponse(bool, bool, bool, bool),

    /// Client is disconnecting
    Disconnect(bool, bool, bool, bool),
}

impl PacketType {
    pub fn to_u8(&self) -> u8 {
        macro_rules! make_type {
            ($typeval:expr, $flag1:expr, $flag2:expr, $flag3:expr, $flag4:expr)
                => (($typeval << 4)
                        | (($flag1 as u8) << 3)
                        | (($flag2 as u8) << 2)
                        | (($flag3 as u8) << 1)
                        | ($flag4 as u8))
        }

        match *self {
            PacketType::Connect(f1, f2, f3, f4) => make_type!(value::CONNECT, f1, f2, f3, f4),
            PacketType::ConnectAcknowledgement(f1, f2, f3, f4) => make_type!(value::CONNACK, f1, f2, f3, f4),

            PacketType::Publish(f1, f2, f3, f4) => make_type!(value::PUBLISH, f1, f2, f3, f4),
            PacketType::PublishAcknowledgement(f1, f2, f3, f4) => make_type!(value::PUBACK, f1, f2, f3, f4),
            PacketType::PublishReceived(f1, f2, f3, f4) => make_type!(value::PUBREC, f1, f2, f3, f4),
            PacketType::PublishRelease(f1, f2, f3, f4) => make_type!(value::PUBREL, f1, f2, f3, f4),
            PacketType::PublishComplete(f1, f2, f3, f4) => make_type!(value::PUBCOMP, f1, f2, f3, f4),

            PacketType::Subscribe(f1, f2, f3, f4) => make_type!(value::SUBSCRIBE, f1, f2, f3, f4),
            PacketType::SubscribeAcknowledgement(f1, f2, f3, f4) => make_type!(value::SUBACK, f1, f2, f3, f4),

            PacketType::Unsubscribe(f1, f2, f3, f4) => make_type!(value::UNSUBSCRIBE, f1, f2, f3, f4),
            PacketType::UnsubscribeAcknowledgement(f1, f2, f3, f4) => make_type!(value::UNSUBACK, f1, f2, f3, f4),

            PacketType::PingRequest(f1, f2, f3, f4) => make_type!(value::PINGREQ, f1, f2, f3, f4),
            PacketType::PingResponse(f1, f2, f3, f4) => make_type!(value::PINGRESP, f1, f2, f3, f4),

            PacketType::Disconnect(f1, f2, f3, f4) => make_type!(value::DISCONNECT, f1, f2, f3, f4),
        }
    }

    pub fn from_u8(val: u8) -> Result<PacketType, PacketTypeError> {

        let type_val = val >> 4;
        let flag = val & 0x0F;

        macro_rules! vconst {
            ($flag:expr, $ret:expr) => (
                if flag != $flag {
                    Err(PacketTypeError::InvalidFlag)
                } else {
                    Ok($ret)
                }
            )
        }

        match type_val {
            value::CONNECT      => vconst!(0x00, PacketType::Connect(false, false, false, false)),
            value::CONNACK      => vconst!(0x00, PacketType::ConnectAcknowledgement(false, false, false, false)),

            value::PUBLISH      => {
                let (f1, f2, f3, f4) = (
                    flag & 0x08 != 0,
                    flag & 0x04 != 0,
                    flag & 0x02 != 0,
                    flag & 0x01 != 0
                );
                Ok(PacketType::Publish(f1, f2, f3, f4))
            },
            value::PUBACK       => vconst!(0x00, PacketType::PublishAcknowledgement(false, false, false, false)),
            value::PUBREC       => vconst!(0x00, PacketType::PublishReceived(false, false, false, false)),
            value::PUBREL       => vconst!(0x02, PacketType::PublishRelease(false, false, true, false)),
            value::PUBCOMP      => vconst!(0x00, PacketType::PublishComplete(false, false, false, false)),

            value::SUBSCRIBE    => vconst!(0x02, PacketType::Subscribe(false, false, true, false)),
            value::SUBACK       => vconst!(0x00, PacketType::SubscribeAcknowledgement(false, false, false, false)),

            value::UNSUBSCRIBE  => vconst!(0x02, PacketType::Unsubscribe(false, false, true, false)),
            value::UNSUBACK     => vconst!(0x00, PacketType::UnsubscribeAcknowledgement(false, false, false, false)),

            value::PINGREQ      => vconst!(0x00, PacketType::PingRequest(false, false, false, false)),
            value::PINGRESP     => vconst!(0x00, PacketType::PingResponse(false, false, false, false)),

            value::DISCONNECT   => vconst!(0x00, PacketType::Disconnect(false, false, false, false)),

            0 | 15              => Err(PacketTypeError::ReservedType(type_val)),
            _                   => Err(PacketTypeError::UndefinedType(type_val)),
        }
    }
}

#[derive(Debug)]
pub enum PacketTypeError {
    ReservedType(u8),
    UndefinedType(u8),
    InvalidFlag,
}

impl fmt::Display for PacketTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PacketTypeError::ReservedType(t) => write!(f, "Reserved type ({})", t),
            &PacketTypeError::UndefinedType(t) => write!(f, "Undefined type ({})", t),
            &PacketTypeError::InvalidFlag => write!(f, "Invalid flag"),
        }
    }
}

impl Error for PacketTypeError {
    fn description(&self) -> &str {
        match self {
            &PacketTypeError::ReservedType(..) => "Reserved type",
            &PacketTypeError::UndefinedType(..) => "Undefined type",
            &PacketTypeError::InvalidFlag => "Invalid flag",
        }
    }
}

mod value {
    pub const CONNECT: u8 = 1;
    pub const CONNACK: u8 = 2;
    pub const PUBLISH: u8 = 3;
    pub const PUBACK: u8 = 4;
    pub const PUBREC: u8 = 5;
    pub const PUBREL: u8 = 6;
    pub const PUBCOMP: u8 = 7;
    pub const SUBSCRIBE: u8 = 8;
    pub const SUBACK: u8 = 9;
    pub const UNSUBSCRIBE: u8 = 10;
    pub const UNSUBACK: u8 = 11;
    pub const PINGREQ: u8 = 12;
    pub const PINGRESP: u8 = 13;
    pub const DISCONNECT: u8 = 14;
}
