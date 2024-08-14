//! Names of metrics we use, to avoid typos

use std::time::{Duration, Instant};

// Re-export everything from `metrics` because we 'stole' its name.
// This way by importing `crate::metrics` we get everything from metrics
// plus our metric names
pub(crate) use metrics::*;

use {
    axum::{extract::MatchedPath, middleware::Next, response::IntoResponse},
    hyper::Request,
    metrics::{describe_gauge, gauge},
    tracing_wrapper::tracing,
};

// General metrics
pub const REQUESTS_COUNTER: &str = "requests_counter";
pub const REQUESTS_DURATION: &str = "http_requests_duration_seconds";

// System metrics
const VIRT_MEM: &str = "process_virtual_memory_bytes";
const RSS_MEM: &str = "process_resident_memory_bytes";
const THREADS: &str = "process_threads";
const OPEN_FDS: &str = "process_open_fds";
const MAX_FDS: &str = "process_max_fds";

/// Spawn a background thread responsible for reporting system metrics
pub fn track_system_metrics() {
    tokio::task::spawn_blocking(|| {
        // Describe metrics
        describe_gauge!(VIRT_MEM, "Virtual memory size in bytes.");
        describe_gauge!(RSS_MEM, "Resident memory size in bytes.");
        describe_gauge!(THREADS, "Number of OS threads in the process.");
        describe_gauge!(OPEN_FDS, "Number of open file descriptors.");
        describe_gauge!(MAX_FDS, "Maximum number of open file descriptors.");

        // Safety: It's always safe to call getpid, sysconf
        let pid = unsafe { libc::getpid() };
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };

        if page_size == -1 {
            tracing::warn!("Unable to get some system info, no cpu/mem info will be gathered");
            return;
        }

        let process = match procfs::process::Process::new(pid) {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!("Unable to construct Process, no cpu/mem info will be gathered");
                return;
            }
        };

        loop {
            // FDs
            if let Ok(fds) = process.fd_count() {
                gauge!(OPEN_FDS, fds as f64);
            }
            if let Ok(limits) = process.limits() {
                if let procfs::process::LimitValue::Value(max) = limits.max_open_files.soft_limit {
                    gauge!(MAX_FDS, max as f64);
                }
            }

            match process.stat() {
                Ok(process_stats) => {
                    // memory
                    let vsize = process_stats.vsize as f64;
                    gauge!(VIRT_MEM, vsize);
                    let rss = (process_stats.rss * page_size as u64) as f64;
                    gauge!(RSS_MEM, rss);

                    // cpu
                    gauge!(THREADS, process_stats.num_threads as f64);
                }
                Err(e) => {
                    tracing::debug!("Failed to get process stats: {:?}", e)
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

/// A service that wraps requests and reports metrics, like latency and count of requests
pub async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    increment_counter!(REQUESTS_COUNTER, &labels);
    histogram!(REQUESTS_DURATION, latency, &labels);

    response
}
