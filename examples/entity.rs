use bytes::BytesMut;
use once_cell::sync::Lazy;
use open_dis_rust::{
    common::{
        GenericHeader, Pdu, PduHeader,
        enums::{PduType, Reason},
    },
    simulation_management::{AcknowledgePdu, StartResumePdu, StopFreezePdu},
};
use std::{net::UdpSocket, sync::Mutex};

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

fn operate() {}

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
    Operating,
    Standby,
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:3000")?;
    println!("Listening on {}", socket.local_addr()?);

    let mut buf = [0u8; 1024];

    loop {
        let (len, _) = socket.recv_from(&mut buf)?;

        if len > 0 {
            let mut bytes = BytesMut::from(&buf[..]);

            let pdu_header = PduHeader::deserialize(&mut bytes);

            match pdu_header.pdu_type {
                PduType::StartResume => {
                    let pdu = StartResumePdu::deserialize_without_header(&mut bytes, pdu_header)
                        .map_err(|e| {
                            eprintln!("Error deserializing StartResumePdu: {}", e);
                            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data")
                        });
                    if pdu.is_ok() {
                        initialize();
                    }
                }
                PduType::StopFreeze => {
                    let pdu = StopFreezePdu::deserialize_without_header(&mut bytes, pdu_header)
                        .map_err(|e| {
                            eprintln!("Error deserializing StopFreezePdu: {}", e);
                            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data")
                        });
                    if pdu.is_ok() {
                        let reason = pdu?.reason;
                        match reason {
                            Reason::Termination => shutdown(),
                            Reason::Recess => standby(),
                            _ => {}
                        }
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }
}
