use bytes::BytesMut;
use once_cell::sync::Lazy;
use open_dis_rust::{
    common::{
        GenericHeader, Pdu, PduHeader,
        data_types::EntityId,
        enums::{PduType, Reason},
    },
    simulation_management::{AcknowledgePdu, StartResumePdu, StopFreezePdu},
};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Mutex,
};

static APP_DATA: Lazy<Mutex<AppData>> = Lazy::new(|| {
    Mutex::new(AppData {
        state: State::Preinit,
    })
});

struct AppData {
    state: State,
}

#[inline]
fn initialize() {
    let mut app_data = APP_DATA.lock().unwrap();

    match app_data.state {
        State::Preinit => {
            println!("Initializing...");
            app_data.state = State::Initialized;
        }
        State::Initialized => {
            println!("Already initialized");
        }
        _ => {
            println!("Invalid state transition");
        }
    }
}

// fn operate() {}

#[inline]
fn standby() {
    let mut app_data = APP_DATA.lock().unwrap();

    match app_data.state {
        State::Initialized => {
            println!("Entering standby mode...");
            app_data.state = State::Standby;
        }
        _ => {
            println!("Invalid state transition");
        }
    }
}

fn shutdown() {
    println!("Shutting down...");
    std::process::exit(0);
}

#[derive(PartialEq, Eq)]
enum State {
    Preinit,
    Initialized,
    //  Operating,
    Standby,
}

fn send_acknowledgement(
    src: SocketAddr,
    incoming_entity_id: EntityId,
    incoming_request_id: u32,
    socket: &UdpSocket,
) -> std::io::Result<()> {
    // Send AcknowledgePdu
    let mut outgoing_pdu = AcknowledgePdu::new();
    outgoing_pdu.acknowledge_flag = open_dis_rust::common::enums::AcknowledgeFlag::StartResume;
    outgoing_pdu.response_flag =
        open_dis_rust::common::enums::AcknowledgeResponseFlag::AbleToComply;

    outgoing_pdu
        .originating_entity_id
        .simulation_address
        .site_id = 1;
    outgoing_pdu
        .originating_entity_id
        .simulation_address
        .application_id = 10;
    outgoing_pdu.originating_entity_id.entity_id = 1;

    outgoing_pdu.receiving_entity_id.simulation_address.site_id =
        incoming_entity_id.simulation_address.site_id;
    outgoing_pdu
        .receiving_entity_id
        .simulation_address
        .application_id = incoming_entity_id.simulation_address.application_id;
    outgoing_pdu.receiving_entity_id.entity_id = incoming_entity_id.entity_id;

    outgoing_pdu.request_id = incoming_request_id;

    let mut outgoing_bytes = BytesMut::new();
    outgoing_pdu
        .serialize(&mut outgoing_bytes)
        .map_err(|_| std::io::ErrorKind::InvalidData)?;

    socket.send_to(&outgoing_bytes, src).map_err(|e| {
        eprintln!("Error deserializing StartResumePdu: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data")
    })?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:3000")?;
    println!("Listening on {}", socket.local_addr()?);

    let mut buf = [0u8; 1024];

    loop {
        let (len, src) = socket.recv_from(&mut buf)?;

        if len > 0 {
            let mut bytes = BytesMut::from(&buf[..]);

            let pdu_header = PduHeader::deserialize(&mut bytes);

            match pdu_header.pdu_type {
                PduType::StartResume => {
                    let incoming_pdu = StartResumePdu::deserialize_without_header(
                        &mut bytes, pdu_header,
                    )
                    .map_err(|e| {
                        eprintln!("Error deserializing StartResumePdu: {}", e);
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data")
                    });
                    if incoming_pdu.is_ok() {
                        initialize();
                    }

                    let incoming_pdu = incoming_pdu.unwrap();

                    send_acknowledgement(
                        src,
                        incoming_pdu.originating_entity_id,
                        incoming_pdu.request_id,
                        &socket,
                    )?;
                }
                PduType::StopFreeze => {
                    let incoming_pdu = StopFreezePdu::deserialize_without_header(
                        &mut bytes, pdu_header,
                    )
                    .map_err(|e| {
                        eprintln!("Error deserializing StopFreezePdu: {}", e);
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data")
                    });
                    if let Ok(ref pdu) = incoming_pdu {
                        let reason = pdu.reason;
                        match reason {
                            Reason::Termination => shutdown(),
                            Reason::Recess => standby(),
                            _ => {}
                        }
                    }

                    let incoming_pdu = incoming_pdu.unwrap();

                    send_acknowledgement(
                        src,
                        incoming_pdu.originating_entity_id,
                        incoming_pdu.request_id,
                        &socket,
                    )?;
                }
                _ => {
                    continue;
                }
            }
        }
    }
}
