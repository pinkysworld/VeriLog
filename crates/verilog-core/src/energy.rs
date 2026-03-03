/// Energy-aware scheduling primitives.
///
/// The research agenda (R01/R05/R17/...) calls for learned policies. The OSS base edition includes
/// a simple rule-based policy and a trait boundary for advanced policies.

#[derive(Debug, Clone, Copy)]
pub struct EnergyFeatures {
    /// Battery state-of-charge in [0, 1]. If unknown, set to 1.0.
    pub battery_soc: f32,
    /// Device temperature in Celsius. If unknown, set to 25.0.
    pub temp_c: f32,
    /// Recent event entropy estimate in [0, ~log2(K)].
    pub recent_event_entropy: f32,
}

pub trait EnergyPolicy: Send + Sync {
    fn next_interval_ms(&self, features: EnergyFeatures) -> u64;
}

/// Simple rule-based policy:
/// - log less frequently when battery is low or temperature is high
/// - log more frequently when entropy is high
#[derive(Debug, Clone)]
pub struct RuleBasedPolicy {
    pub base_interval_ms: u64,
    pub min_interval_ms: u64,
    pub max_interval_ms: u64,
}

impl Default for RuleBasedPolicy {
    fn default() -> Self {
        Self {
            base_interval_ms: 5_000,
            min_interval_ms: 200,
            max_interval_ms: 60_000,
        }
    }
}

impl EnergyPolicy for RuleBasedPolicy {
    fn next_interval_ms(&self, f: EnergyFeatures) -> u64 {
        let mut interval = self.base_interval_ms as f32;

        // Battery scaling: below 30% slow down.
        if f.battery_soc < 0.3 {
            interval *= 2.5;
        } else if f.battery_soc < 0.6 {
            interval *= 1.3;
        }

        // Thermal scaling: above 70C slow down.
        if f.temp_c > 70.0 {
            interval *= 2.0;
        } else if f.temp_c > 55.0 {
            interval *= 1.3;
        }

        // Entropy scaling: more novel events -> speed up.
        if f.recent_event_entropy > 2.5 {
            interval *= 0.6;
        } else if f.recent_event_entropy > 1.5 {
            interval *= 0.8;
        }

        let interval = interval
            .round()
            .clamp(self.min_interval_ms as f32, self.max_interval_ms as f32);
        interval as u64
    }
}
