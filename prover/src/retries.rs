use anyhow::format_err;
use backoff::future::retry_notify;
use std::future::Future;
use std::time::Duration;
use tracing::warn;

/// Repeats the function execution on the exponential backoff principle.
pub async fn with_retries<I, E, Fn, Fut>(operation: Fn) -> anyhow::Result<I>
where
    Fn: FnMut() -> Fut,
    Fut: Future<Output = Result<I, backoff::Error<E>>>,
    E: std::fmt::Display,
{
    let notify = |err, next_after: Duration| {
        let duration_secs = next_after.as_millis() as f32 / 1000.0f32;
        warn!(
            "Failed to reach server err: <{}>, retrying after: {:.1}s",
            err, duration_secs,
        )
    };

    retry_notify(get_backoff(), operation, notify)
        .await
        .map_err(|e| {
            format_err!(
                "Process exit task, for the max elapsed time of the backoff: {}",
                e
            )
        })
}

/// Returns default prover options for backoff configuration.
fn get_backoff() -> backoff::ExponentialBackoff {
    backoff::ExponentialBackoff {
        current_interval: Duration::from_secs(5),
        initial_interval: Duration::from_secs(5),
        multiplier: 1.5f64,
        max_interval: Duration::from_secs(30),
        max_elapsed_time: Some(Duration::from_secs(2 * 60)),
        ..Default::default()
    }
}
