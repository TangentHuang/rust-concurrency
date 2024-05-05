mod matrix;
mod metrics;
mod vector;

pub use matrix::{multiply, Matrix};
pub use metrics::amap::AmapMetrics;
pub use metrics::cmap::CmapMetrics;
pub use vector::{dot_product, Vector};
