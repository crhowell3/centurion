use open_dis_rust::common::data_types::EntityId;
use tauri::State;

use std::io;
use std::net::UdpSocket;
use std::sync::{MutexGuard, RwLock};

use anyhow::Result;

use bytes::BytesMut;
use open_dis_rust::common::enums::{ActionRequestActionID, PduType, Reason};
use open_dis_rust::common::{GenericHeader, Pdu, PduHeader};
use open_dis_rust::simulation_management::{
    AcknowledgePdu, ActionRequestPdu, ActionResponsePdu, StartResumePdu, StopFreezePdu,
};

use crate::config::AppConfig;
use crate::core::RequestIds;
use crate::core::app_state::AppState;

fn handle_res(socket: &UdpSocket) -> Result<(), String> {
    // Wait for response
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;

    tracing::debug!("Received a response with length {len}");

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

fn initialize_cmd(
    socket: UdpSocket,
    mut ids: MutexGuard<'_, RequestIds>,
    dest_addr: String,
) -> Result<(), String> {
    let mut bytes = BytesMut::new();
    let mut pdu = ActionRequestPdu::new();

    let site_id: u16 = 1;
    let app_id: u16 = 50;
    let entity_id: u16 = 1;

    let centurion_id = EntityId::new(site_id, app_id, entity_id);
    let receive_all: EntityId = EntityId::new(0xFFFF, 0xFFFF, 0xFFFF);

    pdu.originating_entity_id = centurion_id;
    pdu.receiving_entity_id = receive_all;
    pdu.action_id = ActionRequestActionID::InitializeInternalParameters as u32;

    pdu.request_id = ids.action_request;

    ids.action_request += 1;

    let _ = pdu
        .serialize(&mut bytes)
        .map_err(|_| io::ErrorKind::InvalidData);

    socket
        .send_to(&bytes, dest_addr)
        .map_err(|e| e.to_string())?;

    handle_res(&socket)?;

    Ok(())
}

fn startup_cmd(
    socket: UdpSocket,
    mut ids: MutexGuard<'_, RequestIds>,
    dest_addr: String,
) -> Result<(), String> {
    let mut bytes = BytesMut::new();
    let mut pdu = StartResumePdu::new();

    let site_id: u16 = 1;
    let app_id: u16 = 50;
    let entity_id: u16 = 1;

    let centurion_id = EntityId::new(site_id, app_id, entity_id);
    let receive_all: EntityId = EntityId::new(0xFFFF, 0xFFFF, 0xFFFF);

    pdu.originating_entity_id = centurion_id;
    pdu.receiving_entity_id = receive_all;

    pdu.request_id = ids.start_resume;

    ids.start_resume += 1;

    let _ = pdu
        .serialize(&mut bytes)
        .map_err(|_| io::ErrorKind::InvalidData);

    socket
        .send_to(&bytes, dest_addr)
        .map_err(|e| e.to_string())?;

    handle_ack(&socket)?;

    Ok(())
}

fn terminate_cmd(
    socket: UdpSocket,
    mut ids: MutexGuard<'_, RequestIds>,
    dest_addr: String,
) -> Result<(), String> {
    let mut bytes = BytesMut::new();
    let mut pdu = StopFreezePdu::new();

    let site_id: u16 = 1;
    let app_id: u16 = 50;
    let entity_id: u16 = 1;

    let centurion_id = EntityId::new(site_id, app_id, entity_id);
    let receive_all: EntityId = EntityId::new(0xFFFF, 0xFFFF, 0xFFFF);

    pdu.originating_entity_id = centurion_id;
    pdu.receiving_entity_id = receive_all;

    pdu.reason = Reason::Termination;

    pdu.request_id = ids.stop_freeze;

    ids.stop_freeze += 1;

    let _ = pdu
        .serialize(&mut bytes)
        .map_err(|_| io::ErrorKind::InvalidData);

    socket
        .send_to(&bytes, dest_addr)
        .map_err(|e| e.to_string())?;

    handle_ack(&socket)?;

    Ok(())
}

fn standby_cmd(
    socket: UdpSocket,
    mut ids: MutexGuard<'_, RequestIds>,
    dest_addr: String,
) -> Result<(), String> {
    let mut bytes = BytesMut::new();
    let mut pdu = StopFreezePdu::new();

    let site_id: u16 = 1;
    let app_id: u16 = 50;
    let entity_id: u16 = 1;

    let centurion_id = EntityId::new(site_id, app_id, entity_id);
    let receive_all: EntityId = EntityId::new(0xFFFF, 0xFFFF, 0xFFFF);

    pdu.originating_entity_id = centurion_id;
    pdu.receiving_entity_id = receive_all;
    pdu.reason = Reason::Recess;

    pdu.request_id = ids.stop_freeze;

    ids.stop_freeze += 1;

    let _ = pdu
        .serialize(&mut bytes)
        .map_err(|_| io::ErrorKind::InvalidData);

    socket
        .send_to(&bytes, dest_addr)
        .map_err(|e| e.to_string())?;

    handle_ack(&socket)?;

    Ok(())
}

fn reset_cmd(
    socket: UdpSocket,
    mut ids: MutexGuard<'_, RequestIds>,
    dest_addr: String,
) -> Result<(), String> {
    let mut bytes = BytesMut::new();
    let mut pdu = StopFreezePdu::new();

    let site_id: u16 = 1;
    let app_id: u16 = 50;
    let entity_id: u16 = 1;

    let centurion_id = EntityId::new(site_id, app_id, entity_id);
    let receive_all: EntityId = EntityId::new(0xFFFF, 0xFFFF, 0xFFFF);

    pdu.originating_entity_id = centurion_id;
    pdu.receiving_entity_id = receive_all;

    pdu.reason = Reason::StopForRestart;

    pdu.request_id = ids.stop_freeze;

    ids.stop_freeze += 1;

    let _ = pdu
        .serialize(&mut bytes)
        .map_err(|_| io::ErrorKind::InvalidData);

    socket
        .send_to(&bytes, dest_addr)
        .map_err(|e| e.to_string())?;

    handle_ack(&socket)?;

    Ok(())
}

#[tauri::command]
pub async fn send_siman_pdu(
    state: State<'_, AppState>,
    config: State<'_, RwLock<AppConfig>>,
    command: String,
) -> Result<(), String> {
    let config = config.read().unwrap();

    // Modifying UDP socket settings using values from scenario configuration file
    let enable_broadcast = config.scenario_config.network.enable_broadcast.clone();
    tracing::trace!("enable_broadcast={enable_broadcast}");
    let multicast_ttl = config.scenario_config.network.multicast_ttl.clone();
    tracing::trace!("multicast_ttl={multicast_ttl}");
    let interface_ip = config.scenario_config.network.interface_ip.clone();
    tracing::trace!("interface_ip={interface_ip}");
    let interface_port = config.scenario_config.network.interface_port.clone();
    tracing::trace!("interface_port={interface_port}");

    let interface_addr = format!("{interface_ip}:{interface_port}");

    let socket = UdpSocket::bind(interface_addr).map_err(|e| e.to_string())?;
    socket
        .set_broadcast(enable_broadcast)
        .unwrap_or_else(|_| tracing::error!("unable to configure UDP broadcast"));
    socket
        .set_multicast_ttl_v4(multicast_ttl)
        .unwrap_or_else(|_| tracing::error!("unable to configure multicast TTL"));

    let ip = config.scenario_config.network.destination_ip.clone();
    tracing::trace!("destination_ip={ip}");
    let port = config.scenario_config.network.destination_port.clone();
    tracing::trace!("destination_port={port}");

    let dest_addr = format!("{ip}:{port}");

    // This is for updating the global request ID count
    let ids = state
        .request_ids
        .lock()
        .map_err(|_| "AppData lock poisoned")?;

    match command.as_str() {
        "initialize" => initialize_cmd(socket, ids, dest_addr),
        "startup" => startup_cmd(socket, ids, dest_addr),
        "terminate" => terminate_cmd(socket, ids, dest_addr),
        "standby" => standby_cmd(socket, ids, dest_addr),
        "reset" => reset_cmd(socket, ids, dest_addr),
        _ => {
            tracing::error!(
                "Received an invalid command from the frontend: {}. This should be impossible... \
                if you see this log, please report this as a bug on https://github.com/crhowell3/centurion",
                command.as_str()
            );
            Err("Invalid command".to_string())
        }
    }
}
