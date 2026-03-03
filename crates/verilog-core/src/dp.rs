use rand::Rng;

use crate::error::VeriLogError;

/// Simple token-bucket privacy budget manager.
///
/// - `epsilon_capacity`: max ε in the bucket
/// - `epsilon_available`: current ε available
/// - `refill_per_sec`: optional replenishment rate (0 = no refill)
#[derive(Debug, Clone)]
pub struct PrivacyBudget {
    pub epsilon_capacity: f64,
    pub epsilon_available: f64,
    pub refill_per_sec: f64,
    pub last_refill_unix_ms: u64,
}

impl PrivacyBudget {
    pub fn new(epsilon_capacity: f64, refill_per_sec: f64, now_unix_ms: u64) -> Self {
        Self {
            epsilon_capacity,
            epsilon_available: epsilon_capacity,
            refill_per_sec,
            last_refill_unix_ms: now_unix_ms,
        }
    }

    pub fn refresh(&mut self, now_unix_ms: u64) {
        if self.refill_per_sec <= 0.0 {
            self.last_refill_unix_ms = now_unix_ms;
            return;
        }
        let dt_ms = now_unix_ms.saturating_sub(self.last_refill_unix_ms) as f64;
        let dt_sec = dt_ms / 1000.0;
        let add = dt_sec * self.refill_per_sec;
        self.epsilon_available = (self.epsilon_available + add).min(self.epsilon_capacity);
        self.last_refill_unix_ms = now_unix_ms;
    }

    pub fn try_spend(&mut self, epsilon: f64, now_unix_ms: u64) -> bool {
        self.refresh(now_unix_ms);
        if self.epsilon_available >= epsilon {
            self.epsilon_available -= epsilon;
            true
        } else {
            false
        }
    }
}

/// Sample Laplace(0, b) noise using inverse CDF.
/// b = sensitivity / epsilon
pub fn laplace_noise<R: Rng + ?Sized>(rng: &mut R, b: f64) -> f64 {
    // Uniform in (-0.5, 0.5)
    let u: f64 = rng.gen_range(-0.5..0.5);
    let sign = if u < 0.0 { -1.0 } else { 1.0 };
    let x = 1.0 - 2.0 * u.abs();
    // avoid ln(0)
    let x = x.max(f64::MIN_POSITIVE);
    sign * b * x.ln() * (-1.0)
}

/// Apply Laplace DP to a numeric value.
pub fn apply_laplace_dp<R: Rng + ?Sized>(
    rng: &mut R,
    value: f64,
    epsilon: f64,
    sensitivity: f64,
) -> Result<f64, VeriLogError> {
    if epsilon <= 0.0 {
        return Err(VeriLogError::Format("epsilon must be > 0".into()));
    }
    if sensitivity < 0.0 {
        return Err(VeriLogError::Format("sensitivity must be >= 0".into()));
    }
    let b = sensitivity / epsilon;
    Ok(value + laplace_noise(rng, b))
}

/// A minimal schema for a numeric telemetry metric event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricEvent {
    pub name: String,
    pub value: f64,
}

/// Apply DP to a JSON-encoded MetricEvent payload.
/// Returns the updated JSON bytes.
pub fn dp_metric_event_json(
    payload_json: &[u8],
    epsilon: f64,
    sensitivity: f64,
    budget: &mut PrivacyBudget,
    now_unix_ms: u64,
) -> Result<Vec<u8>, VeriLogError> {
    if !budget.try_spend(epsilon, now_unix_ms) {
        return Err(VeriLogError::Integrity(
            "privacy budget exhausted for this event".into(),
        ));
    }

    let mut evt: MetricEvent = serde_json::from_slice(payload_json)?;
    let mut rng = rand::thread_rng();
    evt.value = apply_laplace_dp(&mut rng, evt.value, epsilon, sensitivity)?;
    let out = serde_json::to_vec(&evt)?;
    Ok(out)
}
