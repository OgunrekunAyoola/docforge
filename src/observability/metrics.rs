use anyhow::Result;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

pub fn init_prometheus() -> Result<PrometheusHandle> {
    let handle = PrometheusBuilder::new().install_recorder()?;
    Ok(handle)
}
