#[cfg(test)]
mod tests {
    use crate::{GitProtocol, ProtocolHandler};
    
    #[test]
    fn test_protocol_handler() {
        let protocol = ProtocolHandler::new();
        
        // Test pkt-line parsing
        let pkt_data = b"0006a\n0000";
        let lines = protocol.parse_pkt_line(pkt_data).unwrap();
        assert_eq!(lines, vec!["a"]);
        
        // Test pkt-line creation
        let lines = vec!["hello"];
        let pkt_data = protocol.create_pkt_line(&lines);
        assert!(pkt_data.starts_with(b"000a"));
        assert!(pkt_data.ends_with(b"0000"));
    }

    #[test]
    fn test_ref_advertisement() {
        let protocol = ProtocolHandler::new();
        
        let refs = vec![
            ("refs/heads/main".to_string(), "1234567890abcdef".repeat(2).chars().take(40).collect()),
            ("refs/heads/develop".to_string(), "abcdef1234567890".repeat(2).chars().take(40).collect()),
        ];
        
        let capabilities = vec!["multi_ack", "side-band-64k"];
        let advertisement = protocol.create_ref_advertisement(&refs, &capabilities);
        
        // Should contain the refs and capabilities
        assert!(!advertisement.is_empty());
    }
}