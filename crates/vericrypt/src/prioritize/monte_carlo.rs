use crate::types::ShapleyApproximationMetadata;

/// Compute Monte Carlo Shapley approximation with dynamic convergence tracking.
pub fn compute_monte_carlo_metadata(
    samples: u64,
    convergence_error: f64,
    converged: bool,
) -> ShapleyApproximationMetadata {
    let confidence_interval = if samples > 0 {
        1.96 * convergence_error / (samples as f64).sqrt()
    } else {
        0.0
    };

    ShapleyApproximationMetadata {
        samples,
        convergence_error,
        confidence_interval,
        converged,
        convergence_threshold: 0.01,
    }
}
