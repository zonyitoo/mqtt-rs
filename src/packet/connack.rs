use std::io::Read;


use control::{FixedHeader, VariableHeader, PacketType, ControlType};
use control::variable_header::{ConnackFlags, ConnectReturnCode};
use packet::{Packet, PacketError};
use Decodable;

#[derive(Debug, Eq, PartialEq)]
pub struct ConnackPacket {
    fixed_header: FixedHeader,
    variable_headers: Vec<VariableHeader>,
    payload: (),
}

impl ConnackPacket {
    pub fn new(session_present: bool, ret_code: u8) -> ConnackPacket {
        ConnackPacket {
            fixed_header: FixedHeader::new(PacketType::with_default(ControlType::ConnectAcknowledgement), 2),
            variable_headers: vec![
                VariableHeader::new(ConnackFlags { session_present: session_present }),
                VariableHeader::new(ConnectReturnCode(ret_code)),
            ],
            payload: (),
        }
    }

    pub fn session_present(&self) -> bool {
        match self.variable_headers[0] {
            VariableHeader::ConnackFlags(flags) => flags.session_present,
            _ => panic!("Could not find Connack Flags in variable header"),
        }
    }

    pub fn return_code(&self) -> u8 {
        match self.variable_headers[0] {
            VariableHeader::ConnectReturnCode(code) => code.0,
            _ => panic!("Could not find Connack Flags in variable header"),
        }
    }

    pub fn set_session_present(&mut self, session_present: bool) {
        match &mut self.variable_headers[0] {
            &mut VariableHeader::ConnackFlags(ref mut flags) => flags.session_present = session_present,
            _ => panic!("Could not find Connack Flags in variable header"),
        }
    }

    pub fn set_return_code(&mut self, code: u8) {
        match &mut self.variable_headers[0] {
            &mut VariableHeader::ConnectReturnCode(ref mut c) => c.0 = code,
            _ => panic!("Could not find Connack Flags in variable header"),
        }
    }
}

impl<'a> Packet<'a> for ConnackPacket {
    type Payload = ();

    fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    fn variable_headers(&self) -> &[VariableHeader] {
        &self.variable_headers[..]
    }

    fn payload(&self) -> &Self::Payload {
        &self.payload
    }

    fn decode_packet<R: Read>(reader: &mut R, fixed_header: FixedHeader) -> Result<Self, PacketError<'a, Self>> {
        let flags: ConnackFlags = try!(Decodable::decode(reader));
        let code: ConnectReturnCode = try!(Decodable::decode(reader));

        Ok(ConnackPacket {
            fixed_header: fixed_header,
            variable_headers: vec![
                VariableHeader::new(flags),
                VariableHeader::new(code),
            ],
            payload: (),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use {Encodable, Decodable};

    #[test]
    pub fn test_connack_packet_basic() {
        let packet = ConnackPacket::new(false, 1);

        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();

        let mut decode_buf = Cursor::new(buf);
        let decoded = ConnackPacket::decode(&mut decode_buf).unwrap();

        assert_eq!(packet, decoded);
    }
}
