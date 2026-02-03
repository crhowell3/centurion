use tauri::State;

use std::io;
use std::net::UdpSocket;

use bytes::BytesMut;
use open_dis_rust::common::enums::{PduType, Reason};
use open_dis_rust::common::{GenericHeader, Pdu, PduHeader};
use open_dis_rust::simulation_management::{AcknowledgePdu, StartResumePdu, StopFreezePdu};

use crate::core::app_state::AppState;

#[tauri::command]
pub fn send_siman_pdu(state: State<AppState>, command: String) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::new();

    match command.as_str() {
        "startup" => {
            let mut pdu = StartResumePdu::new();

            pdu.originating_entity_id.simulation_address.application_id = 50;
            pdu.originating_entity_id.simulation_address.site_id = 1;
            pdu.originating_entity_id.entity_id = 1;

            pdu.receiving_entity_id.simulation_address.application_id = 65535;
            pdu.receiving_entity_id.simulation_address.site_id = 65535;
            pdu.receiving_entity_id.entity_id = 65535;

            let mut ids = state
                .request_ids
                .lock()
                .map_err(|_| "AppData lock poisoned")?;

            pdu.request_id = ids.start_resume;

            ids.start_resume += 1;

            let _ = pdu
                .serialize(&mut bytes)
                .map_err(|_| io::ErrorKind::InvalidData);

            socket
                .send_to(&bytes, "127.0.0.1:3000")
                .map_err(|e| e.to_string())?;

            // Wait for response
            let mut buf = [0; 1024];
            loop {
                let (len, _) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
                if len > 0 {
                    let mut bytes = BytesMut::from(&buf[..]);
                    let pdu_header = PduHeader::deserialize(&mut bytes);

                    match pdu_header.pdu_type {
                        PduType::Acknowledge => {
                            let ack_pdu =
                                AcknowledgePdu::deserialize_without_header(&mut bytes, pdu_header)
                                    .map_err(|e| {
                                        eprintln!("Error deserializing AcknowledgePdu: {}", e);
                                        std::io::Error::new(
                                            std::io::ErrorKind::InvalidData,
                                            "Invalid data",
                                        )
                                    });
                            let _ = dbg!(ack_pdu);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        "terminate" => {
            let mut pdu = StopFreezePdu::new();

            pdu.originating_entity_id.simulation_address.application_id = 50;
            pdu.originating_entity_id.simulation_address.site_id = 1;
            pdu.originating_entity_id.entity_id = 1;

            pdu.receiving_entity_id.simulation_address.application_id = 65535;
            pdu.receiving_entity_id.simulation_address.site_id = 65535;
            pdu.receiving_entity_id.entity_id = 65535;

            pdu.reason = Reason::Termination;

            let mut ids = state
                .request_ids
                .lock()
                .map_err(|_| "AppData lock poisoned")?;

            pdu.request_id = ids.stop_freeze;

            ids.stop_freeze += 1;

            let _ = pdu
                .serialize(&mut bytes)
                .map_err(|_| io::ErrorKind::InvalidData);

            socket
                .send_to(&bytes, "127.0.0.1:3000")
                .map_err(|e| e.to_string())?;
        }
        "standby" => {
            let mut pdu = StopFreezePdu::new();

            pdu.originating_entity_id.simulation_address.application_id = 50;
            pdu.originating_entity_id.simulation_address.site_id = 1;
            pdu.originating_entity_id.entity_id = 1;

            pdu.receiving_entity_id.simulation_address.application_id = 65535;
            pdu.receiving_entity_id.simulation_address.site_id = 65535;
            pdu.receiving_entity_id.entity_id = 65535;

            pdu.reason = Reason::Recess;

            let mut ids = state
                .request_ids
                .lock()
                .map_err(|_| "AppData lock poisoned")?;

            pdu.request_id = ids.stop_freeze;

            ids.stop_freeze += 1;

            let _ = pdu
                .serialize(&mut bytes)
                .map_err(|_| io::ErrorKind::InvalidData);

            socket
                .send_to(&bytes, "127.0.0.1:3000")
                .map_err(|e| e.to_string())?;
        }
        "reset" => {
            let mut pdu = StopFreezePdu::new();

            pdu.originating_entity_id.simulation_address.application_id = 50;
            pdu.originating_entity_id.simulation_address.site_id = 1;
            pdu.originating_entity_id.entity_id = 1;

            pdu.receiving_entity_id.simulation_address.application_id = 65535;
            pdu.receiving_entity_id.simulation_address.site_id = 65535;
            pdu.receiving_entity_id.entity_id = 65535;

            pdu.reason = Reason::StopForRestart;

            let mut ids = state
                .request_ids
                .lock()
                .map_err(|_| "AppData lock poisoned")?;

            pdu.request_id = ids.stop_freeze;

            ids.stop_freeze += 1;

            let _ = pdu
                .serialize(&mut bytes)
                .map_err(|_| io::ErrorKind::InvalidData);

            socket
                .send_to(&bytes, "127.0.0.1:3000")
                .map_err(|e| e.to_string())?;
        }
        _ => return Err("Invalid command".to_string()),
    }

    Ok(())
}
