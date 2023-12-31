mod settings;

use settings::Settings;

use axum::{
    extract::Query,
    http::{header, Method},
    routing::get,
    routing::IntoMakeService,
    Json, Router, Server,
};
use hyper::server::conn::AddrIncoming;
use log::info;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

trait PrintRandomStuffs {
    fn print_random_stuffs(&self);
}

impl PrintRandomStuffs for MainResponse {
    fn print_random_stuffs(&self) {
        println!("Full Name: {} {}", self.first_name, self.last_name);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MainResponse {
    first_name: String,
    last_name: String,
}
use opentelemetry::sdk::trace;
use opentelemetry::sdk::trace::{RandomIdGenerator, Sampler};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Deserialize, Debug)]
struct Remarks {
    remarks: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new().expect("Failed to load settings");

    println!("Initialising OTEL with {}", &settings.otel.url);

    // OpenTelemetry Trace
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(settings.otel.url),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "ote-axum-testing",
                )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Couldn't create OTLP tracer");

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::try_new("INFO").expect("Failed to initialise tracer"))
        .with(tracing_subscriber::fmt::layer()) // std logs
        .with(telemetry_layer) // otel
        .init();

    // fire off
    let server_handle = start_server();
    let _ = tokio::join!(server_handle.await);

    Ok(())
}

async fn get_name(pagination: Query<Remarks>) -> Json<MainResponse> {
    let remarks: Remarks = pagination.0;

    let name = MainResponse {
        first_name: "Test Person Firstname".to_string(),
        last_name: "Test Person Lastname".to_string(),
    };

    name.print_random_stuffs();
    make_name(&name).unwrap();

    Json(name)
}

#[tracing::instrument]
fn make_name(name: &MainResponse) -> Result<()> {
    let j = serde_json::to_string(&name)?;

    println!("{}", j);
    info!("Name has been created");

    Ok(())
}

async fn start_server() -> Server<AddrIncoming, IntoMakeService<Router>> {
    let cors = CorsLayer::new()
        // allow `GET` and `POST`
        .allow_methods(vec![Method::GET, Method::POST])
        // Allow content-type header - unsure if this replaces defaults?
        .allow_headers([header::CONTENT_TYPE])
        // allow requests from any origin
        .allow_origin(Any);

    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors);

    let app = Router::new().route("/", get(get_name)).layer(service);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    info!("listening on {}", addr);
    info!("What is going on??!?!");
    axum::Server::bind(&addr).serve(app.into_make_service())
}
