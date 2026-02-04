use open_dis_rust::common::data_types::EntityId;
use tauri::State;

use std::io;
use std::net::UdpSocket;

use anyhow::Result;

use bytes::BytesMut;
use open_dis_rust::common::enums::{PduType, Reason, ActionRequestActionID};
use open_dis_rust::common::{GenericHeader, Pdu, PduHeader};
use open_dis_rust::simulation_management::{AcknowledgePdu, ActionRequestPdu, ActionResponsePdu, StartResumePdu, StopFreezePdu};

use crate::core::app_state::AppState;

fn handle_res(socket: &UdpSocket) -> Result<(), String> {
    // Wait for response
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::from(&buf[..len]);
    let pdu_header = PduHeader::deserialize(&mut bytes);

    if pdu_header.pdu_type != PduType::ActionResponse {
        return Err("unexpected PDU type received in response".into());
    }

    ActionResponsePdu::deserialize_without_header(&mut bytes, pdu_header)
        .map_err(|e| format!("ActionResponsePdu deserialization error: {e}"))?;

    Ok(())
}

fn handle_ack(socket: &UdpSocket) -> Result<(), String> {
    // Wait for response
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::from(&buf[..len]);
    let pdu_header = PduHeader::deserialize(&mut bytes);

    if pdu_header.pdu_type != PduType::Acknowledge {
        return Err("unexpected PDU type received in response".into());
    }

    AcknowledgePdu::deserialize_without_header(&mut bytes, pdu_header)
        .map_err(|e| format!("AcknowledgePdu deserialization error: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn send_siman_pdu(state: State<AppState>, command: String) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::new();

    let centurion_id = EntityId::new(1, 50, 1);
    let receive_all: EntityId = EntityId::new(0xFFFF, 0xFFFF, 0xFFFF);

    match command.as_str() {
        "initialize" => {
            let mut pdu = ActionRequestPdu::new();

            pdu.originating_entity_id = centurion_id;
            pdu.receiving_entity_id = receive_all;
            pdu.action_id = ActionRequestActionID::Initializeinternalparameters as u32;

            let mut ids = state.request_ids.lock().map_err(|_| "AppData lock poisoned")?;

            pdu.request_id = ids.action_request;

            ids.action_request += 1;

            let _ = pdu
                .serialize(&mut bytes)
                .map_err(|_| io::ErrorKind::InvalidData);

            socket
                .send_to(&bytes, "127.0.0.1:3000")
                .map_err(|e| e.to_string())?;

            handle_res(&socket)?;

            Ok(())
        }
        "startup" => {
            let mut pdu = StartResumePdu::new();

            pdu.originating_entity_id = centurion_id;
            pdu.receiving_entity_id = receive_all;

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

            handle_ack(&socket)?;

            Ok(())
        }
        "terminate" => {
            let mut pdu = StopFreezePdu::new();

            pdu.originating_entity_id = centurion_id;
            pdu.receiving_entity_id = receive_all;

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

            handle_ack(&socket)?;

            Ok(())
        }
        "standby" => {
            let mut pdu = StopFreezePdu::new();

            pdu.originating_entity_id = centurion_id;
            pdu.receiving_entity_id = receive_all;
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

            handle_ack(&socket)?;

            Ok(())
        }
        "reset" => {
            let mut pdu = StopFreezePdu::new();

            pdu.originating_entity_id = centurion_id;
            pdu.receiving_entity_id = receive_all;

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

            handle_ack(&socket)?;

            Ok(())
        }
        _ => Err("Invalid command".to_string()),
    }
}
