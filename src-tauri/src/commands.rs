use std::io;
use std::net::UdpSocket;

use bytes::BytesMut;
use open_dis_rust::common::Pdu;
use open_dis_rust::common::enums::Reason;
use open_dis_rust::simulation_management::{StartResumePdu, StopFreezePdu};

use crate::app_data::AppData;

#[tauri::command]
pub fn send_startup(state: tauri::State<AppData>) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::new();
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

    Ok(())
}

#[tauri::command]
pub fn send_terminate(state: tauri::State<AppData>) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::new();
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

    Ok(())
}

#[tauri::command]
pub fn send_standby(state: tauri::State<AppData>) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::new();
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

    Ok(())
}

#[tauri::command]
pub fn send_restart(state: tauri::State<AppData>) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;

    let mut bytes = BytesMut::new();
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

    Ok(())
}
