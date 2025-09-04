use crate::{GitObject, ObjectType, PackEntry};
use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;
use nom::{
    bytes::complete::tag,
    number::complete::{be_u32, u8},
    IResult,
};
use std::io::Read;

/// Git pack file header
#[derive(Debug)]
pub struct PackHeader {
    pub signature: [u8; 4], // "PACK"
    pub version: u32,       // Usually 2
    pub num_objects: u32,
}

/// Git pack file parser
pub struct PackParser;

impl PackParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse pack file header
    pub fn parse_header<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], PackHeader> {
        let (input, signature) = tag(b"PACK")(input)?;
        let (input, version) = be_u32(input)?;
        let (input, num_objects) = be_u32(input)?;

        Ok((
            input,
            PackHeader {
                signature: [signature[0], signature[1], signature[2], signature[3]],
                version,
                num_objects,
            },
        ))
    }

    /// Parse a single object from pack data
    pub fn parse_object<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], PackEntry> {
        let (input, type_and_size) = self.parse_type_and_size(input)?;
        let obj_type = self.get_object_type(type_and_size.0)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;
        let size = type_and_size.1;

        // Read compressed data (simplified - in real implementation would need to handle deltas)
        let (input, compressed_data) = self.read_compressed_data(input, size)?;

        // Decompress the data
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut data = Vec::new();
        decoder
            .read_to_end(&mut data)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;

        Ok((
            input,
            PackEntry {
                object_type: obj_type,
                size,
                data,
            },
        ))
    }

    fn parse_type_and_size<'a>(&self, input: &'a [u8]) -> IResult<&'a [u8], (u8, usize)> {
        let (mut input, first_byte) = u8(input)?;
        let obj_type = (first_byte >> 4) & 0x07;
        let mut size = (first_byte & 0x0f) as usize;
        let mut shift = 4;

        // Continue reading size bytes if MSB is set
        while (first_byte & 0x80) != 0 {
            let (remaining, size_byte) = u8(input)?;
            input = remaining;
            size |= ((size_byte & 0x7f) as usize) << shift;
            shift += 7;
            if (size_byte & 0x80) == 0 {
                break;
            }
        }

        Ok((input, (obj_type, size)))
    }

    fn get_object_type(&self, type_id: u8) -> Result<ObjectType> {
        match type_id {
            1 => Ok(ObjectType::Commit),
            2 => Ok(ObjectType::Tree),
            3 => Ok(ObjectType::Blob),
            4 => Ok(ObjectType::Tag),
            _ => Err(anyhow!("Unknown object type: {}", type_id)),
        }
    }

    fn read_compressed_data<'a>(&self, input: &'a [u8], _size: usize) -> IResult<&'a [u8], Vec<u8>> {
        // Simplified - in real implementation would properly parse compressed stream
        // For now, just return the remaining data
        Ok((&[], input.to_vec()))
    }

    /// Create a pack file from objects
    pub fn create_pack(&self, objects: &[GitObject]) -> Result<Vec<u8>> {
        let mut pack_data = Vec::new();

        // Write pack header
        pack_data.extend_from_slice(b"PACK");
        pack_data.extend_from_slice(&2u32.to_be_bytes()); // version
        pack_data.extend_from_slice(&(objects.len() as u32).to_be_bytes());

        // Write objects (simplified)
        for obj in objects {
            let type_id = match obj.obj_type {
                ObjectType::Commit => 1u8,
                ObjectType::Tree => 2u8,
                ObjectType::Blob => 3u8,
                ObjectType::Tag => 4u8,
            };

            // Write type and size (simplified encoding)
            let first_byte = (type_id << 4) | (obj.size & 0x0f) as u8;
            pack_data.push(first_byte);

            // Write compressed content (simplified - should use zlib)
            pack_data.extend_from_slice(&obj.content);
        }

        Ok(pack_data)
    }
}

impl Default for PackParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pack_header_parsing() {
        let parser = PackParser::new();
        
        // Test header parsing
        let header_data = b"PACK\x00\x00\x00\x02\x00\x00\x00\x01"; // version 2, 1 object
        let (_, header) = parser.parse_header(header_data).unwrap();
        
        assert_eq!(&header.signature, b"PACK");
        assert_eq!(header.version, 2);
        assert_eq!(header.num_objects, 1);
    }
    
    #[test]
    fn test_object_type_parsing() {
        let parser = PackParser::new();
        
        assert_eq!(parser.get_object_type(1).unwrap(), ObjectType::Commit);
        assert_eq!(parser.get_object_type(2).unwrap(), ObjectType::Tree);
        assert_eq!(parser.get_object_type(3).unwrap(), ObjectType::Blob);
        assert_eq!(parser.get_object_type(4).unwrap(), ObjectType::Tag);
        
        assert!(parser.get_object_type(99).is_err());
    }
}