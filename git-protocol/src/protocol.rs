use crate::{GitObject, GitProtocol, PackEntry};
use anyhow::{anyhow, Result};
use std::str;

/// Git protocol handler implementing the Git wire protocol
pub struct ProtocolHandler;

impl ProtocolHandler {
    pub fn new() -> Self {
        Self
    }

    /// Parse capabilities from the first pkt-line
    pub fn parse_capabilities(&self, line: &str) -> (String, Vec<String>) {
        if let Some(null_pos) = line.find('\0') {
            let (ref_part, caps_part) = line.split_at(null_pos);
            let capabilities: Vec<String> = caps_part[1..]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            (ref_part.to_string(), capabilities)
        } else {
            (line.to_string(), Vec::new())
        }
    }

    /// Create a reference advertisement
    pub fn create_ref_advertisement(&self, refs: &[(String, String)], capabilities: &[&str]) -> Vec<u8> {
        let mut lines = Vec::new();
        
        if refs.is_empty() {
            // Send null ref with capabilities if no refs exist
            let caps_str = capabilities.join(" ");
            lines.push(format!("0000000000000000000000000000000000000000 capabilities^{}\0{}", "{}", caps_str));
        } else {
            // Send first ref with capabilities
            let caps_str = capabilities.join(" ");
            lines.push(format!("{} {}\0{}", refs[0].1, refs[0].0, caps_str));
            
            // Send remaining refs
            for (ref_name, ref_hash) in refs.iter().skip(1) {
                lines.push(format!("{} {}", ref_hash, ref_name));
            }
        }

        self.create_pkt_line(&lines.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    /// Parse want/have lines from upload-pack request
    pub fn parse_want_have(&self, pkt_lines: &[String]) -> Result<(Vec<String>, Vec<String>)> {
        let mut wants = Vec::new();
        let mut haves = Vec::new();

        for line in pkt_lines {
            let line = line.trim();
            if line.starts_with("want ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    wants.push(parts[1].to_string());
                }
            } else if line.starts_with("have ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    haves.push(parts[1].to_string());
                }
            }
        }

        Ok((wants, haves))
    }

    /// Create NAK response
    pub fn create_nak(&self) -> Vec<u8> {
        self.create_pkt_line(&["NAK"])
    }

    /// Create ACK response
    pub fn create_ack(&self, hash: &str) -> Vec<u8> {
        self.create_pkt_line(&[&format!("ACK {}", hash)])
    }
}

impl GitProtocol for ProtocolHandler {
    fn parse_pack(&self, data: &[u8]) -> Result<Vec<PackEntry>> {
        let parser = crate::pack::PackParser::new();
        let (remaining, header) = parser
            .parse_header(data)
            .map_err(|e| anyhow!("Failed to parse pack header: {:?}", e))?;

        let mut entries = Vec::new();
        let mut current = remaining;

        for _ in 0..header.num_objects {
            let (remaining, entry) = parser
                .parse_object(current)
                .map_err(|e| anyhow!("Failed to parse pack object: {:?}", e))?;
            entries.push(entry);
            current = remaining;
        }

        Ok(entries)
    }

    fn create_pack(&self, objects: &[GitObject]) -> Result<Vec<u8>> {
        let parser = crate::pack::PackParser::new();
        parser.create_pack(objects)
    }

    fn parse_pkt_line(&self, data: &[u8]) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            if pos + 4 > data.len() {
                break;
            }

            // Read length prefix (4 hex digits)
            let length_str = str::from_utf8(&data[pos..pos + 4])
                .map_err(|e| anyhow!("Invalid UTF-8 in length prefix: {}", e))?;
            let length = u16::from_str_radix(length_str, 16)
                .map_err(|e| anyhow!("Invalid hex length: {}", e))?;

            if length == 0 {
                // Flush packet
                pos += 4;
                break;
            }

            if length < 4 {
                return Err(anyhow!("Invalid packet length: {}", length));
            }

            let content_length = (length - 4) as usize;
            if pos + 4 + content_length > data.len() {
                return Err(anyhow!("Packet extends beyond data"));
            }

            let content = str::from_utf8(&data[pos + 4..pos + 4 + content_length])
                .map_err(|e| anyhow!("Invalid UTF-8 in packet content: {}", e))?;
            
            lines.push(content.trim_end_matches('\n').to_string());
            pos += 4 + content_length;
        }

        Ok(lines)
    }

    fn create_pkt_line(&self, lines: &[&str]) -> Vec<u8> {
        let mut result = Vec::new();

        for line in lines {
            let content_length = line.len() + 1; // +1 for newline
            let total_length = content_length + 4; // +4 for length prefix
            
            // Write length prefix as 4-digit hex
            result.extend_from_slice(format!("{:04x}", total_length).as_bytes());
            
            // Write content with newline
            result.extend_from_slice(line.as_bytes());
            result.push(b'\n');
        }

        // Add flush packet (0000)
        result.extend_from_slice(b"0000");
        result
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}