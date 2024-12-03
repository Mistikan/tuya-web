use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Router,
};
use clap::{CommandFactory, Parser};
use log::warn;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use outlet::Outlet;
use std::iter::zip;
use std::net::IpAddr;
use std::sync::Arc;

mod outlet;

/// Web API for controlling Tuya outlets
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Device name
    #[arg(short, long)]
    name: Vec<String>,

    /// Device ID
    #[arg(short, long)]
    device_ids: Vec<String>,

    /// Local key
    #[arg(short, long)]
    key: Vec<String>,

    /// IP address
    #[arg(short, long)]
    address: Vec<IpAddr>,

    /// Version
    #[arg(short, long)]
    protocol_version: Vec<String>,
}

#[derive(Clone)]
struct AppState {
    outlets: Arc<Vec<Outlet>>,
    recorder_handle: Arc<PrometheusHandle>
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    // Check that length of arrays are equal
    if !(args.device_ids.len() == args.key.len() && args.key.len() == args.address.len()) {
        Args::command()
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                "Must be same number of local keys, device ids, and addresses",
            )
            .exit();
    }

    let outlets: Vec<_> = zip(zip(zip(zip(args.device_ids, args.key), args.address), args.protocol_version), args.name)
        .map(|((((dev_id, key), address), protocol_version), name)| Outlet {
            name,
            dev_id,
            key,
            address,
            protocol_version
        })
        .collect();

    let recorder_handle = setup_metrics_recorder();

    let state = AppState {
        outlets: Arc::new(outlets),
        recorder_handle: Arc::new(recorder_handle),
    };

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/outlet/:outlet_id", post(toggle_outlet))
        .route("/outlet/:outlet_id/:outlet_state", put(set_outlet))
        .route("/outlet/:outlet_id", get(get_outlet))
        .route("/metrics", get(get_metrics))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_metrics(
    State(app_state): State<AppState>
) -> (StatusCode, String) {
    // Set flag tuya_smartplug_last_scrape_error
    let mut tuya_smartplug_last_scrape_error = false; 

    // Set metric tuya_smartplug_count_devices
    let tuya_smartplug_count_devices = app_state.outlets.len();
    metrics::gauge!("tuya_smartplug_count_devices").set(tuya_smartplug_count_devices as u8);

    // Set device metrics
    for outlet in app_state.outlets.iter() {
        let device = outlet.name.clone();
        match outlet.metrics().await {
            Ok(dps) => {
                metrics::gauge!("tuya_smartplug_current", "device" => device.clone()).set(f64::from(dps.current)/1000.0);
                metrics::gauge!("tuya_smartplug_power", "device" => device.clone()).set(f64::from(dps.power)/100.0);
                metrics::gauge!("tuya_smartplug_voltage", "device" => device.clone()).set(f64::from(dps.voltage)/100.0);
                metrics::gauge!("tuya_smartplug_frequency", "device" => device.clone()).set(f64::from(dps.frequency)/100.0);

            },
            Err(e) => {
                tuya_smartplug_last_scrape_error = true;
                warn!("{}", e);
            },
        };
    }

    // Set global metrics
    metrics::gauge!("tuya_smartplug_last_scrape_error").set(tuya_smartplug_last_scrape_error as i8);
    metrics::counter!("tuya_smartplug_scrapes_total").increment(1);

    // Return metrics
    let metrics = app_state.recorder_handle.render();
    return (StatusCode::OK, metrics);
}

fn setup_metrics_recorder() -> PrometheusHandle {
    // https://github.com/tokio-rs/axum/blob/main/examples/prometheus-metrics/src/main.rs
    // https://ellie.wtf/notes/exporting-prometheus-metrics-with-axum
    // Ищи metrics::counter! и дергай эндпоинт, чтобы метрика появилась
    PrometheusBuilder::new()
        .install_recorder()
        .unwrap()
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "hellu :3"
}

async fn get_outlet(
    State(app_state): State<AppState>,
    Path((outlet_id,)): Path<(usize,)>,
) -> (StatusCode, String) {
    if let Some(outlet) = app_state.outlets.get(outlet_id) {
        match outlet.get().await {
            Ok(true) => (StatusCode::OK, "true".into()),
            Ok(false) => (StatusCode::OK, "false".into()),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                format!("failed to get got err: {}", e),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, "no such outlet".into())
    }
}

async fn set_outlet(
    State(app_state): State<AppState>,
    Path((outlet_id, state)): Path<(usize, bool)>,
) -> (StatusCode, String) {
    if let Some(outlet) = app_state.outlets.get(outlet_id) {
        match outlet.set(state).await {
            Ok(_) => (StatusCode::OK, "ok".into()),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                format!("failed to set got err: {}", e),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, "no such outlet".into())
    }
}

async fn toggle_outlet(
    State(app_state): State<AppState>,
    Path((outlet_id,)): Path<(usize,)>,
) -> (StatusCode, String) {
    if let Some(outlet) = app_state.outlets.get(outlet_id) {
        match outlet.toggle().await {
            Ok(_) => (StatusCode::OK, "ok".into()),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                format!("failed to toggle got err: {}", e),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, "no such outlet".into())
    }
}
