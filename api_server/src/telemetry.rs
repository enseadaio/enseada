use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry::sdk::Resource;
use opentelemetry::{global, KeyValue};

pub fn init_exporter() -> PrometheusExporter {
    let exporter = opentelemetry_prometheus::exporter()
        .with_resource(Resource::new(vec![]))
        .init();

    let meter = global::meter("enseada");
    let build_info = meter.u64_counter("enseada_build_info")
        .with_description("Enseada build information")
        .init();
    build_info.add(1, &[
        KeyValue::new("version", env!("CARGO_PKG_VERSION")),
    ]);

    exporter
}
