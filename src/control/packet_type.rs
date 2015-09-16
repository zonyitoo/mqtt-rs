use std::error::Error;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PacketType {
    pub control_type: ControlType,
    pub flags: u8,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ControlType {
    /// Client request to connect to Server
    Connect                         = value::CONNECT,

    /// Connect acknowledgment
    ConnectAcknowledgement          = value::CONNACK,

    /// Publish message
    Publish                         = value::PUBLISH,

    /// Publish acknowledgment
    PublishAcknowledgement          = value::PUBACK,

    /// Publish received (assured delivery part 1)
    PublishReceived                 = value::PUBREC,

    /// Publish release (assured delivery part 2)
    PublishRelease                  = value::PUBREL,

    /// Publish complete (assured delivery part 3)
    PublishComplete                 = value::PUBCOMP,

    /// Client subscribe request
    Subscribe                       = value::SUBSCRIBE,

    /// Subscribe acknowledgment
    SubscribeAcknowledgement        = value::SUBACK,

    /// Unsubscribe request
    Unsubscribe                     = value::UNSUBSCRIBE,

    /// Unsubscribe acknowledgment
    UnsubscribeAcknowledgement      = value::UNSUBACK,

    /// PING request
    PingRequest                     = value::PINGREQ,

    /// PING response
    PingResponse                    = value::PINGRESP,

    /// Client is disconnecting
    Disconnect                      = value::DISCONNECT,
}

impl PacketType {
    #[inline]
    pub fn new(t: ControlType, flags: u8) -> PacketType {
        PacketType {
            control_type: t,
            flags: flags,
        }
    }

    #[inline]
    pub fn with_default(t: ControlType) -> PacketType {
        match t {
            ControlType::Connect => PacketType::new(t, 0),
            ControlType::ConnectAcknowledgement => PacketType::new(t, 0),

            ControlType::Publish => PacketType::new(t, 0),
            ControlType::PublishAcknowledgement => PacketType::new(t, 0),
            ControlType::PublishReceived => PacketType::new(t, 0),
            ControlType::PublishRelease => PacketType::new(t, 0x02),
            ControlType::PublishComplete => PacketType::new(t, 0),

            ControlType::Subscribe => PacketType::new(t, 0x02),
            ControlType::SubscribeAcknowledgement => PacketType::new(t, 0),

            ControlType::Unsubscribe => PacketType::new(t, 0x02),
            ControlType::UnsubscribeAcknowledgement => PacketType::new(t, 0),

            ControlType::PingRequest => PacketType::new(t, 0),
            ControlType::PingResponse => PacketType::new(t, 0),

            ControlType::Disconnect => PacketType::new(t, 0),
        }
    }

    pub fn to_u8(&self) -> u8 {
        (self.control_type as u8) << 4
            | (self.flags & 0x0F)
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
            value::CONNECT      => vconst!(0x00, PacketType::new(ControlType::Connect,
                                           0)),
            value::CONNACK      => vconst!(0x00, PacketType::new(ControlType::ConnectAcknowledgement,
                                           0)),

            value::PUBLISH      =>
                Ok(PacketType::new(ControlType::Publish, flag & 0x0F)),
            value::PUBACK       => vconst!(0x00, PacketType::new(ControlType::PublishAcknowledgement,
                                                                 0)),
            value::PUBREC       => vconst!(0x00, PacketType::new(ControlType::PublishReceived,
                                                                 0)),
            value::PUBREL       => vconst!(0x02, PacketType::new(ControlType::PublishRelease,
                                                                 0x02)),
            value::PUBCOMP      => vconst!(0x00, PacketType::new(ControlType::PublishComplete,
                                                                 0)),

            value::SUBSCRIBE    => vconst!(0x02, PacketType::new(ControlType::Subscribe, 0x02)),
            value::SUBACK       => vconst!(0x00, PacketType::new(ControlType::SubscribeAcknowledgement,
                                                                 0)),

            value::UNSUBSCRIBE  => vconst!(0x02, PacketType::new(ControlType::Unsubscribe, 0x02)),
            value::UNSUBACK     => vconst!(0x00, PacketType::new(ControlType::UnsubscribeAcknowledgement,
                                                                 0)),

            value::PINGREQ      => vconst!(0x00, PacketType::new(ControlType::PingRequest, 0)),
            value::PINGRESP     => vconst!(0x00, PacketType::new(ControlType::PingResponse,
                                                                 0)),

            value::DISCONNECT   => vconst!(0x00, PacketType::new(ControlType::Disconnect, 0)),

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
