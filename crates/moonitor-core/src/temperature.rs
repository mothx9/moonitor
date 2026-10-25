//! Interpretation of raw thermal observations. Wire samples remain intact.
use crate::TemperatureSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalKind {
    ApplePmu,
    HostComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalValidity {
    Valid,
    NotFinite,
    BelowAbsoluteZero,
    OutsideSensorRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermalObservation<'a> {
    pub label: &'a str,
    pub raw_celsius: f32,
    pub value_celsius: Option<f32>,
    pub validity: ThermalValidity,
    pub kind: ThermalKind,
    pub source: &'static str,
    pub sampled_at_unix_ms: u64,
}

impl ThermalObservation<'_> {
    pub fn reason(&self) -> &'static str {
        match self.validity {
            ThermalValidity::Valid => "Lettura valida",
            ThermalValidity::NotFinite => "Valore termico non finito",
            ThermalValidity::BelowAbsoluteZero => "Valore inferiore allo zero assoluto",
            ThermalValidity::OutsideSensorRange => {
                "Fuori dall'intervallo del sensore PMU (−40…150 °C)"
            }
        }
    }
}

impl TemperatureSnapshot {
    /// PMU component policy is separate from general component readings: genuine
    /// subzero readings remain valid. A source anomaly invalidates only its field.
    pub fn observation(&self, sampled_at_unix_ms: u64) -> ThermalObservation<'_> {
        let kind = if self.label.starts_with("PMU ") {
            ThermalKind::ApplePmu
        } else {
            ThermalKind::HostComponent
        };
        let validity = if !self.celsius.is_finite() {
            ThermalValidity::NotFinite
        } else if self.celsius < -273.15 {
            ThermalValidity::BelowAbsoluteZero
        } else if kind == ThermalKind::ApplePmu && !(-40.0..=150.0).contains(&self.celsius) {
            ThermalValidity::OutsideSensorRange
        } else {
            ThermalValidity::Valid
        };
        ThermalObservation {
            label: &self.label,
            raw_celsius: self.celsius,
            value_celsius: (validity == ThermalValidity::Valid).then_some(self.celsius),
            validity,
            kind,
            source: "sysinfo · component temperature",
            sampled_at_unix_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_pmu_anomalies_are_quarantined_without_losing_raw_evidence() {
        let raw: Vec<TemperatureSnapshot> =
            serde_json::from_str(include_str!("../tests/fixtures/macos-pmu-anomalies.json"))
                .unwrap();
        for t in raw {
            let o = t.observation(1234);
            assert_eq!(o.raw_celsius, t.celsius);
            assert_eq!(o.sampled_at_unix_ms, 1234);
            assert_eq!(o.value_celsius.is_some(), t.celsius > -100.0);
            if t.celsius < -100.0 {
                assert_eq!(o.validity, ThermalValidity::BelowAbsoluteZero);
            }
        }
    }

    #[test]
    fn zero_subzero_missing_and_pmu_limits_are_not_conflated() {
        for (label, raw, valid) in [
            ("ambient", -80.0, true),
            ("PMU tdie", -20.0, true),
            ("PMU tdie", 0.0, true),
            ("PMU tdie", -40.0, true),
            ("PMU tdie", 150.0, true),
            ("PMU tdie", 151.0, false),
            ("ambient", -274.0, false),
            ("ambient", f32::NAN, false),
            ("ambient", f32::INFINITY, false),
        ] {
            let o = TemperatureSnapshot {
                label: label.into(),
                celsius: raw,
            }
            .observation(0)
            .value_celsius;
            assert_eq!(o.is_some(), valid, "{label}: {raw}");
        }
    }
}
