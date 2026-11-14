use std::time::{Duration, SystemTime};

use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

use crate::config::RETRY_AFTER_MS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryAfter {
    Unsaid,
    Wait(Duration),
}

impl RetryAfter {
    pub fn read(headers: &HeaderMap) -> RetryAfter {
        let mut advice = RetryAfter::Unsaid;
        for value in headers.get_all(RETRY_AFTER) {
            advice = seconds_or_date(value);
        }
        for value in headers.get_all(RETRY_AFTER_MS) {
            advice = milliseconds(value);
        }
        advice
    }
}

fn milliseconds(value: &HeaderValue) -> RetryAfter {
    let text = match value.to_str() {
        Ok(text) => text.trim(),
        Err(error) => {
            tracing::warn!(%error, "retry-after-ms is not text");
            return RetryAfter::Unsaid;
        }
    };
    match text.parse::<f64>() {
        Ok(ms) => match Duration::try_from_secs_f64(ms / 1000.0) {
            Ok(wait) => RetryAfter::Wait(wait),
            Err(error) => {
                tracing::warn!(%error, "retry-after-ms is out of range");
                RetryAfter::Unsaid
            }
        },
        Err(error) => {
            tracing::warn!(%error, "retry-after-ms is not a number");
            RetryAfter::Unsaid
        }
    }
}

fn seconds_or_date(value: &HeaderValue) -> RetryAfter {
    let text = match value.to_str() {
        Ok(text) => text.trim(),
        Err(error) => {
            tracing::warn!(%error, "retry-after is not text");
            return RetryAfter::Unsaid;
        }
    };
    match text.parse::<u64>() {
        Ok(seconds) => RetryAfter::Wait(Duration::from_secs(seconds)),
        Err(number) => match httpdate::parse_http_date(text) {
            Ok(at) => match at.duration_since(SystemTime::now()) {
                Ok(wait) => RetryAfter::Wait(wait),
                Err(error) => {
                    tracing::warn!(%error, "retry-after date has already passed");
                    RetryAfter::Wait(Duration::ZERO)
                }
            },
            Err(date) => {
                tracing::warn!(%number, %date, "retry-after is neither seconds nor a date");
                RetryAfter::Unsaid
            }
        },
    }
}
