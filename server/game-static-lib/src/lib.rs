use std::{sync::Arc, time::Duration};

use globed_game_server::{bridge::CentralBridge, gs_entry_point, server::FfiMessage, state::ServerState, util::TokioChannel, StartupConfiguration};
use globed_shared::{error, info, log, warn, LogLevelFilter, StaticLogger, StaticLoggerCallback, SyncMutex, DEFAULT_GAME_SERVER_PORT};

fn int_to_log_level(log_level: i32) -> LogLevelFilter {
    match log_level {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 => LogLevelFilter::Debug,
        4 => LogLevelFilter::Trace,
        _ => LogLevelFilter::Warn, // default to warn
    }
}

// global sender
static REQUEST_UP_CHANNEL: SyncMutex<Option<Arc<TokioChannel<FfiMessage>>>> = SyncMutex::new(None);
static REQUEST_DOWN_CHANNEL: SyncMutex<Option<std::sync::mpsc::Receiver<FfiMessage>>> = SyncMutex::new(None);

#[repr(C)]
pub struct ServerEntryData {
    log_level: i32,
    callback: Option<StaticLoggerCallback>,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn globed_gsi_entry(data: &ServerEntryData) -> bool {
    // we are not actually 'globed_game_server', but 99% of logs come from there, the exception being the log below
    log::set_logger(StaticLogger::instance("globed_game_server", data.callback)).unwrap();

    log::set_max_level(int_to_log_level(data.log_level));

    warn!("Starting static game server. Hello from Rust :)");

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let startup_config = StartupConfiguration {
        bind_address: format!("0.0.0.0:{}", DEFAULT_GAME_SERVER_PORT).parse().unwrap(),
        central_data: None,
    };

    let state = ServerState::new(&[]);
    let bridge = CentralBridge::new("", "");

    let up = Arc::new(TokioChannel::new(8));
    let (down_tx, down_rx) = std::sync::mpsc::channel();

    REQUEST_UP_CHANNEL.lock().replace(up.clone());
    REQUEST_DOWN_CHANNEL.lock().replace(down_rx);

    match rt.block_on(gs_entry_point(startup_config, state, bridge, true, Some((up, down_tx)))) {
        Ok(()) => true,
        Err(err) => {
            error!("server exited with error: {err}");
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn globed_gsi_is_running() -> bool {
    REQUEST_UP_CHANNEL.lock().is_some()
}

// User requests

#[inline]
fn _handle_req(msg: FfiMessage) -> Option<FfiMessage> {
    let up = REQUEST_UP_CHANNEL.lock();
    let down = REQUEST_DOWN_CHANNEL.lock();

    match (&*up, &*down) {
        (Some(up), Some(down)) => {
            if let Ok(()) = up.try_send(msg) {
                match down.recv_timeout(Duration::from_secs(5)) {
                    Ok(msg) => Some(msg),
                    _ => {
                        error!("Failed to receive response from the server");
                        None
                    }
                }
            } else {
                error!("Failed to send message to server");
                None
            }
        }
        _ => {
            error!("Server is not running");
            None
        }
    }
}

fn _handle_req_no_resp(msg: FfiMessage) -> bool {
    let up = REQUEST_UP_CHANNEL.lock();

    match &*up {
        Some(up) => {
            if let Ok(()) = up.try_send(msg) {
                true
            } else {
                error!("Failed to send message to server");
                false
            }
        }
        _ => {
            error!("Server is not running");
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn globed_gsi_shutdown() -> bool {
    warn!("Shutting down static game server");

    let ret = _handle_req(FfiMessage::Shutdown);

    // cleanup
    match ret {
        Some(FfiMessage::ShutdownAck) => {
            REQUEST_UP_CHANNEL.lock().take();
            REQUEST_DOWN_CHANNEL.lock().take();
            true
        }
        _ => false,
    }
}

#[no_mangle]
pub extern "C" fn globed_gsi_print_status() -> bool {
    _handle_req_no_resp(FfiMessage::PrintInfo)
}
