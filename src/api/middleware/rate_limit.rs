use crate::error::AppError;
use axum::{extract::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use governor::{clock::DefaultClock, state::InMemoryState, Quota, RateLimiter};
use std::{net::IpAddr, num::NonZeroU32, sync::Arc};

type Limiter = RateLimiter<IpAddr, DashMap<IpAddr, InMemoryState>, DefaultClock>;

pub struct IpRateLimiter {
    limiter: Limiter,
}

impl IpRateLimiter {
    pub fn per_second(n: u32) -> Arc<Self> {
        Arc::new(Self {
            limiter: RateLimiter::keyed(Quota::per_second(
                NonZeroU32::new(n).expect("rate must be > 0"),
            )),
        })
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        self.limiter.check_key(&ip).is_ok()
    }
}

/// Extract IP from X-Forwarded-For header, falling back to 0.0.0.0
fn extract_ip(req: &Request) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
}

pub async fn by_ip(
    limiter: Arc<IpRateLimiter>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_ip(&req);
    if !limiter.check(ip) {
        return Err(AppError::TooManyRequests);
    }
    Ok(next.run(req).await)
}
