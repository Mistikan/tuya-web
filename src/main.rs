use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Router,
};
use clap::{CommandFactory, Parser};
use outlet::Outlet;
use std::iter::zip;
use std::net::IpAddr;
use std::sync::Arc;

mod outlet;

/// Web API for controlling Tuya outlets
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Device ID
    #[arg(short, long)]
    device_ids: Vec<String>,

    /// Local key
    #[arg(short, long)]
    key: Vec<String>,

    /// IP address
    #[arg(short, long)]
    address: Vec<IpAddr>,
}

#[derive(Clone)]
struct AppState {
    outlets: Arc<Vec<Outlet>>,
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

    let outlets: Vec<_> = zip(zip(args.device_ids, args.key), args.address)
        .map(|((dev_id, key), address)| Outlet {
            dev_id,
            key,
            address,
        })
        .collect();

    let state = AppState {
        outlets: Arc::new(outlets),
    };

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/outlet/:outlet_id", post(toggle_outlet))
        .route("/outlet/:outlet_id/:outlet_state", put(set_outlet))
        .route("/outlet/:outlet_id", get(get_outlet))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
