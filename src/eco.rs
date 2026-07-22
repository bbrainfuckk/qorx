use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ScenarioFactors {
    pub kwh_per_million_tokens: Option<f64>,
    pub kg_co2e_per_kwh: Option<f64>,
    pub liters_per_kwh: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnvironmentalScenario {
    pub kwh_avoided: Option<f64>,
    pub kg_co2e_avoided: Option<f64>,
    pub liters_water_avoided: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoReport {
    pub schema: String,
    pub qorx_version: String,
    pub local_tokens: u64,
    pub sent_tokens: u64,
    pub tokens_avoided: u64,
    pub reduction_x: Option<f64>,
    pub token_counts_source: String,
    pub factors: ScenarioFactors,
    pub scenario: EnvironmentalScenario,
    pub network_calls: u64,
    pub boundary: String,
}

pub fn build_report(
    local_tokens: u64,
    sent_tokens: u64,
    factors: ScenarioFactors,
) -> Result<EcoReport> {
    validate_factors(factors)?;
    let tokens_avoided = local_tokens.saturating_sub(sent_tokens);
    let kwh_avoided = factors
        .kwh_per_million_tokens
        .map(|factor| tokens_avoided as f64 / 1_000_000.0 * factor);
    let kg_co2e_avoided = kwh_avoided
        .zip(factors.kg_co2e_per_kwh)
        .map(|(kwh, factor)| kwh * factor);
    let liters_water_avoided = kwh_avoided
        .zip(factors.liters_per_kwh)
        .map(|(kwh, factor)| kwh * factor);

    Ok(EcoReport {
        schema: "qorx.eco.v1".to_string(),
        qorx_version: crate::version::QORX_VERSION.to_string(),
        local_tokens,
        sent_tokens,
        tokens_avoided,
        reduction_x: (sent_tokens > 0).then(|| local_tokens as f64 / sent_tokens as f64),
        token_counts_source: "user_supplied".to_string(),
        factors,
        scenario: EnvironmentalScenario {
            kwh_avoided,
            kg_co2e_avoided,
            liters_water_avoided,
        },
        network_calls: 0,
        boundary: "Token counts and environmental factors are user supplied. Energy, CO2e, and water values are optional scenarios, not universal measurements. Results depend on hardware, workload, electricity source, cooling, and reporting boundary. Null means not calculated."
            .to_string(),
    })
}

fn validate_factors(factors: ScenarioFactors) -> Result<()> {
    for (name, value) in [
        ("--kwh-per-million-tokens", factors.kwh_per_million_tokens),
        ("--kg-co2e-per-kwh", factors.kg_co2e_per_kwh),
        ("--liters-per-kwh", factors.liters_per_kwh),
    ] {
        if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
            bail!("{name} must be finite and non-negative");
        }
    }
    if factors.kwh_per_million_tokens.is_none()
        && (factors.kg_co2e_per_kwh.is_some() || factors.liters_per_kwh.is_some())
    {
        bail!(
            "CO2e and water scenarios require --kwh-per-million-tokens so their energy basis is explicit"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_token_math_without_inventing_impact_factors() {
        let report = build_report(13_200_000, 8, ScenarioFactors::default()).expect("report");

        assert_eq!(report.tokens_avoided, 13_199_992);
        assert_eq!(report.reduction_x, Some(1_650_000.0));
        assert_eq!(report.token_counts_source, "user_supplied");
        assert_eq!(report.scenario.kwh_avoided, None);
        assert_eq!(report.scenario.kg_co2e_avoided, None);
        assert_eq!(report.scenario.liters_water_avoided, None);
    }

    #[test]
    fn calculates_only_the_supplied_scenario() {
        let report = build_report(
            2_000_000,
            1_000_000,
            ScenarioFactors {
                kwh_per_million_tokens: Some(0.5),
                kg_co2e_per_kwh: Some(0.4),
                liters_per_kwh: Some(1.2),
            },
        )
        .expect("report");

        assert_eq!(report.scenario.kwh_avoided, Some(0.5));
        assert_eq!(report.scenario.kg_co2e_avoided, Some(0.2));
        assert_eq!(report.scenario.liters_water_avoided, Some(0.6));
    }

    #[test]
    fn rejects_impact_factors_without_an_energy_basis() {
        let result = build_report(
            10,
            1,
            ScenarioFactors {
                kg_co2e_per_kwh: Some(0.4),
                ..ScenarioFactors::default()
            },
        );

        assert!(result.is_err());
    }
}
