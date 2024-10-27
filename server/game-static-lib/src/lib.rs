use globed_game_server::{bridge::CentralBridge, gs_entry_point, state::ServerState, StartupConfiguration};
use globed_shared::{error, DEFAULT_GAME_SERVER_PORT};

#[no_mangle]
pub extern "C" fn gs_static_entry_point() -> bool {
    let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

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
