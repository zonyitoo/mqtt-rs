//! CONNACK

use std::io::Read;

use crate::control::variable_header::{ConnackFlags, ConnectReturnCode};
use crate::control::{ControlType, FixedHeader, PacketType};
use crate::packet::{DecodablePacket, PacketError};
use crate::Decodable;

/// `CONNACK` packet
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ConnackPacket {
    fixed_header: FixedHeader,
    flags: ConnackFlags,
    ret_code: ConnectReturnCode,
}

encodable_packet!(ConnackPacket(flags, ret_code));

impl ConnackPacket {
    pub fn new(session_present: bool, ret_code: ConnectReturnCode) -> ConnackPacket {
        ConnackPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::ConnectAcknowledgement), 2),
            flags: ConnackFlags { session_present },
            ret_code,
        }
    }

    pub fn connack_flags(&self) -> ConnackFlags {
        self.flags
    }

    pub fn connect_return_code(&self) -> ConnectReturnCode {
        self.ret_code
    }
}

impl DecodablePacket for ConnackPacket {
    type Payload = ();

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<Self>> {
        let flags: ConnackFlags = Decodable::decode(reader)?;
        let code: ConnectReturnCode = Decodable::decode(reader)?;

        Ok(ConnackPacket {
            fixed_header,
            flags,
            ret_code: code,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use crate::control::variable_header::ConnectReturnCode;
    use crate::{Decodable, Encodable};

    #[test]
    pub fn test_connack_packet_basic() {
        let packet = ConnackPacket::new(false, ConnectReturnCode::IdentifierRejected);

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        let mut decode_buf = Cursor::new(buf);
        let decoded = ConnackPacket::decode(&mut decode_buf).unwrap();

        assert_eq!(packet, decoded);
    }
}
