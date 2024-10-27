use globed_game_server::{bridge::CentralBridge, gs_entry_point, state::ServerState, StartupConfiguration};
use globed_shared::{error, log, warn, LogLevelFilter, StaticLogger, StaticLoggerCallback, DEFAULT_GAME_SERVER_PORT};

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

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn gs_static_entry_point(log_level: i32, dll_name: *const i8, callback: Option<StaticLoggerCallback>) -> bool {
    // construct &str from the raw pointer
    let dll_name = unsafe { std::ffi::CStr::from_ptr(dll_name).to_str().unwrap() };

    log::set_logger(StaticLogger::instance(dll_name, callback)).unwrap();
    log::set_max_level(int_to_log_level(log_level));

    warn!("Starting static game server. Hello from Rust :)");

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    let startup_config = StartupConfiguration {
        bind_address: format!("0.0.0.0:{}", DEFAULT_GAME_SERVER_PORT).parse().unwrap(),
        central_data: None,
    };

    let state = ServerState::new(&[]);
    let bridge = CentralBridge::new("", "");

    match rt.block_on(gs_entry_point(startup_config, state, bridge, true, false)) {
        Ok(()) => unreachable!("server should never exit"),
        Err(err) => {
            error!("server exited with error: {err}");
            false
        }
    }
}
