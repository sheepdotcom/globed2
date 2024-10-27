#![feature(sync_unsafe_cell, duration_constructors, async_closure, let_chains, if_let_guard)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::wildcard_imports,
    clippy::redundant_closure_for_method_calls
)]

pub mod bridge;
pub mod client;
pub mod data;
pub mod managers;
pub mod server;
pub mod state;
pub mod util;
use std::{error::Error, net::SocketAddr};

pub use bridge::{CentralBridge, CentralBridgeError};
use globed_shared::{debug, error, warn, webhook, DEFAULT_GAME_SERVER_PORT};

// #[cfg(feature = "use_tokio_tracing")]
// use tokio_tracing as tokio;

use server::{FfiChannelPair, GameServer};
pub use state::ServerState;
// #[cfg(not(feature = "use_tokio_tracing"))]
#[allow(clippy::single_component_path_imports)]
use tokio;
use tokio::net::{TcpListener, UdpSocket};

pub struct StartupConfiguration {
    pub bind_address: SocketAddr,
    pub central_data: Option<(String, String)>,
}

pub fn abort_misconfig() -> ! {
    error!("aborting launch due to misconfiguration.");
    std::process::exit(1);
}

fn censor_key(key: &str, keep_first_n_chars: usize) -> String {
    if key.len() <= keep_first_n_chars {
        return "*".repeat(key.len());
    }

    format!("{}{}", &key[..keep_first_n_chars], "*".repeat(key.len() - keep_first_n_chars))
}

pub async fn gs_entry_point(
    startup_config: StartupConfiguration,
    state: ServerState,
    bridge: CentralBridge,
    standalone: bool,
    ffi_channel: Option<FfiChannelPair>,
) -> Result<(), Box<dyn Error>> {
    let is_dedicated = ffi_channel.is_none();

    {
        // output useful information

        let gsbd = bridge.central_conf.lock();

        debug!("Configuration:");
        debug!("* TPS: {}", gsbd.tps);
        debug!("* Token expiry: {} seconds", gsbd.token_expiry);
        debug!("* Maintenance: {}", if gsbd.maintenance { "yes" } else { "no" });

        debug!("* Token secret key: '{}'", censor_key(&gsbd.secret_key2, 4));

        if standalone {
            debug!("* Admin key: '{}'", gsbd.admin_key);
        } else {
            // print first 4 chars, rest is censored
            debug!("* Admin key: '{}'", censor_key(&gsbd.admin_key, 4));
        }

        if gsbd.chat_burst_limit == 0 || gsbd.chat_burst_interval == 0 {
            debug!("* Text chat ratelimit: disabled");
        } else {
            debug!(
                "* Text chat ratelimit: {} messages per {}ms",
                gsbd.chat_burst_limit, gsbd.chat_burst_interval
            );
        }

        state.role_manager.refresh_from(&gsbd);
    }

    // bind the UDP socket

    let udp_socket = match UdpSocket::bind(&startup_config.bind_address).await {
        Ok(x) => x,
        Err(err) => {
            error!("Failed to bind the UDP socket with address {}: {err}", startup_config.bind_address);
            if startup_config.bind_address.port() < 1024 {
                warn!("hint: ports below 1024 are commonly privileged and you can't use them as a regular user");
                warn!("hint: pick a higher port number or leave it out completely to use the default port number ({DEFAULT_GAME_SERVER_PORT})");
            }

            if is_dedicated {
                abort_misconfig();
            } else {
                return Err(Box::new(err));
            }
        }
    };

    // bind the TCP socket

    let tcp_socket = match TcpListener::bind(&startup_config.bind_address).await {
        Ok(x) => x,
        Err(err) => {
            error!("Failed to bind the TCP socket with address {}: {err}", startup_config.bind_address);

            if is_dedicated {
                abort_misconfig();
            } else {
                return Err(Box::new(err));
            }
        }
    };

    // create and run the server

    let server = GameServer::new(tcp_socket, udp_socket, state, bridge, standalone, ffi_channel);
    let server = Box::leak(Box::new(server));

    Box::pin(server.run()).await;

    Ok(())
}
