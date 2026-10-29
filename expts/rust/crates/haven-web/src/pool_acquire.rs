//! Optional SQLx pool acquisition telemetry for isolated performance runs.

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing::span::{self, Id};
use tracing::subscriber::{self, Interest};
use tracing::{Event, Metadata, Subscriber};

struct PoolAcquireSubscriber {
    samples: Arc<Mutex<Vec<f64>>>,
}

struct AcquireDuration(Option<f64>);

impl Visit for AcquireDuration {
    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name() == "acquired_after_secs" {
            self.0 = Some(value);
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}

impl Subscriber for PoolAcquireSubscriber {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if self.enabled(metadata) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == "sqlx::pool::acquire"
    }

    fn new_span(&self, _attrs: &span::Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _span: &Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if event.metadata().target() != "sqlx::pool::acquire" {
            return;
        }
        let mut duration = AcquireDuration(None);
        event.record(&mut duration);
        if let Some(seconds) = duration.0 {
            if let Ok(mut samples) = self.samples.lock() {
                samples.push(seconds);
            }
        }
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(LevelFilter::INFO)
    }
}

pub(super) fn install(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    let samples = Arc::new(Mutex::new(Vec::<f64>::new()));
    let pending = Arc::clone(&samples);
    let _flush_thread = std::thread::Builder::new()
        .name("pool-acquire-telemetry".to_string())
        .spawn(move || {
            let mut output = BufWriter::new(file);
            loop {
                std::thread::sleep(Duration::from_millis(250));
                let batch = pending
                    .lock()
                    .map(|mut values| std::mem::take(&mut *values))
                    .unwrap_or_default();
                if batch.is_empty() {
                    continue;
                }
                for seconds in batch {
                    output
                        .write_fmt(format_args!("{seconds:.9}\n"))
                        .unwrap_or(());
                }
                output.flush().unwrap_or(());
            }
        })?;
    subscriber::set_global_default(Arc::new(PoolAcquireSubscriber { samples }))?;
    Ok(())
}
