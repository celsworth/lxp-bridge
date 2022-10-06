use anyhow::Result;
use log::{error, info};
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() {
    tokio::select! {
        result = lxp_bridge::app() => {
            if let Err(err) = result {
                error!("{:?}", err);
                std::process::exit(255);
            }
        }
        _ = cancel_on_int_or_term() => {
        }
    }
}

/// Provides a future that will terminate once a SIGINT or SIGTERM is
/// received from the host. Allows the process to be terminated
/// cleanly when running in a container (particularly Kubernetes).
async fn cancel_on_int_or_term() -> Result<()> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM, stopping process");
        },
        _ = sigint.recv() => {
            info!("Received SIGINT, stopping process");
        },
    }

    Ok(())
}
