//! Packet types

use crate::qos::QualityOfService;

/// Packet type
// INVARIANT: the high 4 bits of the byte must be a valid control type
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PacketType(u8);

/// Defined control types
#[rustfmt::skip]
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

impl ControlType {
    #[inline]
    fn default_flags(self) -> u8 {
        match self {
            ControlType::Connect => 0,
            ControlType::ConnectAcknowledgement => 0,

            ControlType::Publish => 0,
            ControlType::PublishAcknowledgement => 0,
            ControlType::PublishReceived => 0,
            ControlType::PublishRelease => 0b0010,
            ControlType::PublishComplete => 0,

            ControlType::Subscribe => 0b0010,
            ControlType::SubscribeAcknowledgement => 0,

            ControlType::Unsubscribe => 0b0010,
            ControlType::UnsubscribeAcknowledgement => 0,

            ControlType::PingRequest => 0,
            ControlType::PingResponse => 0,

            ControlType::Disconnect => 0,
        }
    }
}

impl PacketType {
    /// Creates a packet type. Returns None if `flags` is an invalid value for the given
    /// ControlType as defined by the [MQTT spec].
    ///
    /// [MQTT spec]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Table_2.2_-
    pub fn new(t: ControlType, flags: u8) -> Result<PacketType, InvalidFlag> {
        let flags_ok = match t {
            ControlType::Publish => {
                let qos = (flags & 0b0110) >> 1;
                matches!(qos, 0 | 1 | 2)
            }
            _ => t.default_flags() == flags,
        };
        if flags_ok {
            Ok(PacketType::new_unchecked(t, flags))
        } else {
            Err(InvalidFlag(t, flags))
        }
    }

    #[inline]
    fn new_unchecked(t: ControlType, flags: u8) -> PacketType {
        let byte = (t as u8) << 4 | (flags & 0x0F);
        #[allow(unused_unsafe)]
        unsafe {
            // SAFETY: just constructed from a valid ControlType
            PacketType(byte)
        }
    }

    /// Creates a packet type with default flags
    ///
    /// <http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Table_2.2_->
    #[inline]
    pub fn with_default(t: ControlType) -> PacketType {
        let flags = t.default_flags();
        PacketType::new_unchecked(t, flags)
    }

    pub(crate) fn publish(qos: QualityOfService) -> PacketType {
        PacketType::new_unchecked(ControlType::Publish, (qos as u8) << 1)
    }

    #[inline]
    pub(crate) fn update_flags(&mut self, upd: impl FnOnce(u8) -> u8) {
        let flags = upd(self.flags());
        self.0 = (self.0 & !0x0F) | (flags & 0x0F)
    }

    /// To code
    #[inline]
    pub fn to_u8(self) -> u8 {
        self.0
    }

    /// From code
    pub fn from_u8(val: u8) -> Result<PacketType, PacketTypeError> {
        let type_val = val >> 4;
        let flags = val & 0x0F;

        let control_type = get_control_type(type_val).ok_or(PacketTypeError::ReservedType(type_val, flags))?;
        Ok(PacketType::new(control_type, flags)?)
    }

    #[inline]
    pub fn control_type(self) -> ControlType {
        get_control_type(self.0 >> 4).unwrap_or_else(|| {
            // SAFETY: this is maintained by the invariant for PacketType
            unsafe { std::hint::unreachable_unchecked() }
        })
    }

    #[inline]
    pub fn flags(self) -> u8 {
        self.0 & 0x0F
    }
}

#[inline]
fn get_control_type(val: u8) -> Option<ControlType> {
    let typ = match val {
        value::CONNECT => ControlType::Connect,
        value::CONNACK => ControlType::ConnectAcknowledgement,

        value::PUBLISH => ControlType::Publish,
        value::PUBACK => ControlType::PublishAcknowledgement,
        value::PUBREC => ControlType::PublishReceived,
        value::PUBREL => ControlType::PublishRelease,
        value::PUBCOMP => ControlType::PublishComplete,

        value::SUBSCRIBE => ControlType::Subscribe,
        value::SUBACK => ControlType::SubscribeAcknowledgement,

        value::UNSUBSCRIBE => ControlType::Unsubscribe,
        value::UNSUBACK => ControlType::UnsubscribeAcknowledgement,

        value::PINGREQ => ControlType::PingRequest,
        value::PINGRESP => ControlType::PingResponse,

        value::DISCONNECT => ControlType::Disconnect,

        _ => return None,
    };
    Some(typ)
}

/// Parsing packet type errors
#[derive(Debug, thiserror::Error)]
pub enum PacketTypeError {
    #[error("reserved type {0:?} (flags {1:#X})")]
    ReservedType(u8, u8),
    #[error(transparent)]
    InvalidFlag(#[from] InvalidFlag),
}

#[derive(Debug, thiserror::Error)]
#[error("invalid flag for {0:?} ({1:#X})")]
pub struct InvalidFlag(pub ControlType, pub u8);

#[rustfmt::skip]
mod value {
    pub const CONNECT:     u8 = 1;
    pub const CONNACK:     u8 = 2;
    pub const PUBLISH:     u8 = 3;
    pub const PUBACK:      u8 = 4;
    pub const PUBREC:      u8 = 5;
    pub const PUBREL:      u8 = 6;
    pub const PUBCOMP:     u8 = 7;
    pub const SUBSCRIBE:   u8 = 8;
    pub const SUBACK:      u8 = 9;
    pub const UNSUBSCRIBE: u8 = 10;
    pub const UNSUBACK:    u8 = 11;
    pub const PINGREQ:     u8 = 12;
    pub const PINGRESP:    u8 = 13;
    pub const DISCONNECT:  u8 = 14;
}
