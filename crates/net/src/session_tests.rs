#[cfg(test)]
mod tests {
    use config::DeviceConfig;
    use crate::data::{Device, DeviceType};
    use crate::layout::DATA_SIZE;
    use crate::session::{ClientSession, ServerSession};

    #[test]
    fn test_client_and_server_session_type_state_flow() {
        let device = Device::try_new(DeviceType::Worker, "sn-test-01", "worker-01").unwrap();

        // 1. Client starts disconnected -> start_handshake produces packet
        let client_disconnected = ClientSession::new();
        let (client_connecting, handshake_pkt) = client_disconnected.start_handshake(&device, 1);

        // 2. Server receives handshake -> accept_handshake transitions to Registered
        let server_unregistered = ServerSession::new();
        let (mut server_registered, ack_pkt, received_dev) = server_unregistered
            .accept_handshake(&handshake_pkt, 0x12345678, 10)
            .expect("server accepts handshake");

        assert_eq!(received_dev.serial_number().unwrap(), "sn-test-01");
        assert_eq!(server_registered.session_token(), 0x12345678);

        // 3. Client receives handshake ack -> transitions to Established
        let mut client_established = client_connecting
            .receive_handshake_ack(&ack_pkt)
            .expect("client validates ack");
        assert_eq!(client_established.session_token(), 0x12345678);
        assert_eq!(client_established.assigned_game_id(), 10);

        // 4. Client sends heartbeat -> Server processes and returns heartbeat ack
        let heartbeat_pkt = client_established.send_heartbeat();
        let hb = server_registered
            .receive_heartbeat(&heartbeat_pkt)
            .expect("server handles heartbeat");
        assert_eq!(hb.flags(), 0);

        let hb_ack_pkt = server_registered.send_heartbeat_ack(5);
        let decoded_ack = client_established
            .receive_heartbeat_ack(&hb_ack_pkt)
            .expect("client handles heartbeat ack");
        assert_eq!(decoded_ack.latest_game_seq(), 5);

        // 5. Server sends game update -> Client receives
        let game_bytes = [0x55; DATA_SIZE];
        let game_pkt = server_registered.send_game_update(game_bytes);
        let received_game_bytes = client_established
            .receive_game_update(&game_pkt)
            .expect("client handles game update");
        assert_eq!(received_game_bytes, game_bytes);

        // 6. Disconnect resets to initial states
        let _ = client_established.disconnect();
        let _ = server_registered.expire();
    }
}
