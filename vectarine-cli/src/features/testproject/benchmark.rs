use runtime::anyhow;

use super::anyhow::Result;
use std::time::{Duration, Instant};

pub(crate) enum Benchmark {
    Pending(PendingBenchmark),
    Finished(BenchmarkResult),
}

pub(crate) struct PendingBenchmark {
    pub(crate) start_frame: u32,
    pub(crate) start_instant: Instant,
    pub(crate) max_allowed_duration: Option<Duration>,
}

pub(crate) struct BenchmarkResult {
    pub(crate) name: String,
    pub(crate) frame_count: u32,
    pub(crate) duration: Duration,
    pub(crate) max_allowed_duration: Option<Duration>,
}

pub fn parse_duration(duration_str: &str) -> Result<Duration> {
    let s = duration_str.trim();

    let split_idx = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .ok_or_else(|| anyhow::anyhow!("Missing duration unit"))?;

    let (value, unit) = s.split_at(split_idx);

    let value: u64 = value.trim().parse()?;
    let unit = unit.trim();

    let duration = match unit {
        "ns" => Duration::from_nanos(value),
        "us" => Duration::from_millis(value),
        "ms" => Duration::from_millis(value),
        "sec" => Duration::from_secs(value),
        "s" => Duration::from_secs(value),
        _ => anyhow::bail!("Unsupported duration unit: {unit}"),
    };

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("10ms").unwrap(), Duration::from_millis(10));
        assert_eq!(parse_duration("10 ms").unwrap(), Duration::from_millis(10));

        assert_eq!(parse_duration("441 ns").unwrap(), Duration::from_nanos(441));
        assert_eq!(parse_duration("30us").unwrap(), Duration::from_micros(30));
        assert_eq!(parse_duration("30 us").unwrap(), Duration::from_micros(30));

        assert_eq!(parse_duration("30 s").unwrap(), Duration::from_secs(30));

        assert_eq!(
            parse_duration("1.5 ms").unwrap(),
            Duration::from_micros(1500)
        );
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("10").is_err());
        assert!(parse_duration("10m").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("10 xs").is_err());
    }
}
