//! Encodable traits

use std::io::{self, Read, Write};
use std::error::Error;
use std::string::FromUtf8Error;
use std::fmt;
use std::convert::From;
use std::marker::Sized;

use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/// Methods for encoding an Object to bytes according to MQTT specification
pub trait Encodable<'a> {
    type Err: Error + 'a;

    /// Encodes to writer
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err>;
    /// Length of bytes after encoded
    fn encoded_length(&self) -> u32;
}

/// Methods for decoding bytes to an Object according to MQTT specification
pub trait Decodable<'a>: Sized {
    type Err: Error + 'a;
    type Cond;

    /// Decodes object from reader
    fn decode<R: Read>(reader: &mut R) -> Result<Self, Self::Err> {
        Self::decode_with(reader, None)
    }

    /// Decodes object with additional data (or hints)
    fn decode_with<R: Read>(reader: &mut R, cond: Option<Self::Cond>) -> Result<Self, Self::Err>;
}

impl<'a> Encodable<'a> for &'a str {
    type Err = StringEncodeError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), StringEncodeError> {
        assert!(self.as_bytes().len() <= u16::max_value() as usize);

        writer
            .write_u16::<BigEndian>(self.as_bytes().len() as u16)
            .map_err(From::from)
            .and_then(|_| writer.write_all(self.as_bytes()))
            .map_err(StringEncodeError::IoError)
    }

    fn encoded_length(&self) -> u32 {
        2 + self.as_bytes().len() as u32
    }
}

impl<'a> Encodable<'a> for &'a [u8] {
    type Err = io::Error;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u16::<BigEndian>(self.len() as u16)
            .map_err(From::from)
            .and_then(|_| writer.write_all(self))
    }

    fn encoded_length(&self) -> u32 {
        self.len() as u32 + 2
    }
}

impl<'a> Encodable<'a> for String {
    type Err = StringEncodeError;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), StringEncodeError> {
        (&self[..]).encode(writer)
    }

    fn encoded_length(&self) -> u32 {
        (&self[..]).encoded_length()
    }
}

impl<'a> Decodable<'a> for String {
    type Err = StringEncodeError;
    type Cond = ();

    fn decode_with<R: Read>(reader: &mut R, _rest: Option<()>) -> Result<String, StringEncodeError> {
        let len = try!(reader.read_u16::<BigEndian>()) as usize;
        let mut buf = Vec::with_capacity(len);
        unsafe {
            buf.set_len(len);
        }
        try!(reader.read_exact(&mut buf));

        String::from_utf8(buf).map_err(StringEncodeError::FromUtf8Error)
    }
}

impl<'a> Encodable<'a> for Vec<u8> {
    type Err = io::Error;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        (&self[..]).encode(writer)
    }

    fn encoded_length(&self) -> u32 {
        (&self[..]).encoded_length()
    }
}

impl<'a> Decodable<'a> for Vec<u8> {
    type Err = io::Error;
    type Cond = u32;

    fn decode_with<R: Read>(reader: &mut R, length: Option<u32>) -> Result<Vec<u8>, io::Error> {
        match length {
            Some(length) => {
                try!(reader.read_u16::<BigEndian>()); //Throw away the initial bytes specifying length
                let mut buf = Vec::with_capacity(length as usize);
                unsafe {
                    buf.set_len(length as usize);
                }
                try!(reader.read_exact(&mut buf));
                Ok(buf)
            }
            None => {
                let length = try!(reader.read_u16::<BigEndian>());
                let mut buf = Vec::with_capacity(length as usize);
                try!(reader.read_to_end(&mut buf));
                Ok(buf)
            }
        }
    }
}

impl<'a> Encodable<'a> for () {
    type Err = NoError;

    fn encode<W: Write>(&self, _: &mut W) -> Result<(), NoError> {
        Ok(())
    }

    fn encoded_length(&self) -> u32 {
        0
    }
}

impl<'a> Decodable<'a> for () {
    type Err = NoError;
    type Cond = ();

    fn decode_with<R: Read>(_: &mut R, _: Option<()>) -> Result<(), NoError> {
        Ok(())
    }
}

/// Bytes that encoded with length
#[derive(Debug, Eq, PartialEq)]
pub struct VarBytes(pub Vec<u8>);

impl<'a> Encodable<'a> for VarBytes {
    type Err = io::Error;
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), Self::Err> {
        assert!(self.0.len() <= u16::max_value() as usize);
        let len = self.0.len() as u16;
        try!(writer.write_u16::<BigEndian>(len));
        try!(writer.write_all(&self.0));
        Ok(())
    }

    fn encoded_length(&self) -> u32 {
        2 + self.0.len() as u32
    }
}

impl<'a> Decodable<'a> for VarBytes {
    type Err = io::Error;
    type Cond = ();
    fn decode_with<R: Read>(reader: &mut R, _: Option<()>) -> Result<VarBytes, io::Error> {
        let length = try!(reader.read_u16::<BigEndian>()) as usize;
        let mut buf = Vec::with_capacity(length);
        unsafe {
            buf.set_len(length);
        }
        try!(reader.read_exact(&mut buf));
        Ok(VarBytes(buf))
    }
}

/// Error that indicates we won't have any errors
#[derive(Debug)]
pub struct NoError;

impl fmt::Display for NoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No error")
    }
}

impl Error for NoError {
    fn description(&self) -> &str {
        "No error"
    }
}

/// Errors while parsing to a string
#[derive(Debug)]
pub enum StringEncodeError {
    IoError(io::Error),
    FromUtf8Error(FromUtf8Error),
    MalformedData,
}

impl fmt::Display for StringEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &StringEncodeError::IoError(ref err) => err.fmt(f),
            &StringEncodeError::FromUtf8Error(ref err) => err.fmt(f),
            &StringEncodeError::MalformedData => write!(f, "Malformed data"),
        }
    }
}

impl Error for StringEncodeError {
    fn description(&self) -> &str {
        match self {
            &StringEncodeError::IoError(ref err) => err.description(),
            &StringEncodeError::FromUtf8Error(ref err) => err.description(),
            &StringEncodeError::MalformedData => "Malformed data",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &StringEncodeError::IoError(ref err) => Some(err),
            &StringEncodeError::FromUtf8Error(ref err) => Some(err),
            &StringEncodeError::MalformedData => None,
        }
    }
}

impl From<io::Error> for StringEncodeError {
    fn from(err: io::Error) -> StringEncodeError {
        StringEncodeError::IoError(err)
    }
}

impl From<FromUtf8Error> for StringEncodeError {
    fn from(err: FromUtf8Error) -> StringEncodeError {
        StringEncodeError::FromUtf8Error(err)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn varbyte_encode() {
        let test_var = vec![0, 1, 2, 3, 4, 5];
        let bytes = VarBytes(test_var);

        assert_eq!(bytes.encoded_length() as usize, 2 + 6);

        let mut buf = Vec::new();
        bytes.encode(&mut buf).unwrap();

        assert_eq!(&buf, &[0, 6, 0, 1, 2, 3, 4, 5]);

        let mut reader = Cursor::new(buf);
        let decoded = VarBytes::decode(&mut reader).unwrap();

        assert_eq!(decoded, bytes);
    }
}
