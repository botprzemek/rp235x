#[cfg(test)]
mod tests {
    use crate::config_machine::{ConfigMachine, ConfigStorage};
    use crate::data::DeviceConfigPacket;
    use crate::event::ServerEvent;

    struct DummyMemoryStorage {
        saved_sn: [u8; 12],
    }

    impl ConfigStorage for DummyMemoryStorage {
        type Error = &'static str;

        fn persist_config(&mut self, packet: &DeviceConfigPacket) -> Result<(), Self::Error> {
            let sn = packet.serial_number().map_err(|_| "invalid serial")?;
            let bytes = sn.as_bytes();
            let len = bytes.len().min(12);
            self.saved_sn[..len].copy_from_slice(&bytes[..len]);
            Ok(())
        }
    }

    #[test]
    fn test_config_machine_sequence() {
        let machine = ConfigMachine::new();

        // Create a valid DeviceConfigPacket
        let client_pkt = DeviceConfigPacket::try_new("sn-12345678", "worker-01", "MyWifi", "secret123").unwrap();
        let raw_bytes = client_pkt.to_bytes();

        // 1. Inbound packet produces Ack and transitions to PendingWrite
        let (pending_machine, packet, ack) = machine.handle_packet(&raw_bytes).expect("valid config packet");
        assert_eq!(ack.event(), ServerEvent::DeviceConfigAck);

        // 2. Storage persists and transitions back to Idle with Success packet
        let mut storage = DummyMemoryStorage { saved_sn: [0; 12] };
        let (_idle_machine, success) = pending_machine.persist_and_complete(&mut storage, &packet).expect("persisted");
        assert_eq!(success.event(), ServerEvent::DeviceConfigSuccess);
        assert_eq!(&storage.saved_sn[..11], b"sn-12345678");
    }
}
