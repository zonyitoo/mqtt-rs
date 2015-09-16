use std::error::Error;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PacketType {
    control_type: ControlType,
    flags: (bool, bool, bool, bool),
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
    pub fn new(t: ControlType, f1: bool, f2: bool, f3: bool, f4: bool) -> PacketType {
        PacketType {
            control_type: t,
            flags: (f1, f2, f3, f4),
        }
    }

    #[inline]
    pub fn with_default(t: ControlType) -> PacketType {
        match t {
            ControlType::Connect => PacketType::new(t, false, false, false, false),
            ControlType::ConnectAcknowledgement => PacketType::new(t, false, false, false, false),

            ControlType::Publish => PacketType::new(t, false, false, false, false),
            ControlType::PublishAcknowledgement => PacketType::new(t, false, false, false, false),
            ControlType::PublishReceived => PacketType::new(t, false, false, false, false),
            ControlType::PublishRelease => PacketType::new(t, false, false, true, false),
            ControlType::PublishComplete => PacketType::new(t, false, false, false, false),

            ControlType::Subscribe => PacketType::new(t, false, false, true, false),
            ControlType::SubscribeAcknowledgement => PacketType::new(t, false, false, false, false),

            ControlType::Unsubscribe => PacketType::new(t, false, false, true, false),
            ControlType::UnsubscribeAcknowledgement => PacketType::new(t, false, false, false, false),

            ControlType::PingRequest => PacketType::new(t, false, false, false, false),
            ControlType::PingResponse => PacketType::new(t, false, false, false, false),

            ControlType::Disconnect => PacketType::new(t, false, false, false, false),
        }
    }

    #[inline]
    pub fn flags(&self) -> (bool, bool, bool, bool) {
        self.flags
    }

    #[inline]
    pub fn control_type(&self) -> ControlType {
        self.control_type
    }

    pub fn to_u8(&self) -> u8 {
        (self.control_type as u8) << 4
            | (self.flags.0 as u8) << 3
            | (self.flags.1 as u8) << 2
            | (self.flags.2 as u8) << 1
            | (self.flags.3 as u8)
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
                                           false, false, false, false)),
            value::CONNACK      => vconst!(0x00, PacketType::new(ControlType::ConnectAcknowledgement,
                                           false, false, false, false)),

            value::PUBLISH      => {
                let (f1, f2, f3, f4) = (
                    flag & 0x08 != 0,
                    flag & 0x04 != 0,
                    flag & 0x02 != 0,
                    flag & 0x01 != 0
                );
                Ok(PacketType::new(ControlType::Publish, f1, f2, f3, f4))
            },
            value::PUBACK       => vconst!(0x00, PacketType::new(ControlType::PublishAcknowledgement,
                                                                 false, false, false, false)),
            value::PUBREC       => vconst!(0x00, PacketType::new(ControlType::PublishReceived,
                                                                 false, false, false, false)),
            value::PUBREL       => vconst!(0x02, PacketType::new(ControlType::PublishRelease,
                                                                 false, false, true, false)),
            value::PUBCOMP      => vconst!(0x00, PacketType::new(ControlType::PublishComplete,
                                                                 false, false, false, false)),

            value::SUBSCRIBE    => vconst!(0x02, PacketType::new(ControlType::Subscribe, false, false, true, false)),
            value::SUBACK       => vconst!(0x00, PacketType::new(ControlType::SubscribeAcknowledgement,
                                                                 false, false, false, false)),

            value::UNSUBSCRIBE  => vconst!(0x02, PacketType::new(ControlType::Unsubscribe, false, false, true, false)),
            value::UNSUBACK     => vconst!(0x00, PacketType::new(ControlType::UnsubscribeAcknowledgement,
                                                                 false, false, false, false)),

            value::PINGREQ      => vconst!(0x00, PacketType::new(ControlType::PingRequest, false, false, false, false)),
            value::PINGRESP     => vconst!(0x00, PacketType::new(ControlType::PingResponse,
                                                                 false, false, false, false)),

            value::DISCONNECT   => vconst!(0x00, PacketType::new(ControlType::Disconnect, false, false, false, false)),

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
