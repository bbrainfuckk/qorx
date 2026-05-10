use serde::{Deserialize, Serialize};

use crate::{
    b2c_quant::{self, B2cPlan},
    index::RepoIndex,
    judge::{self, JudgeReport},
    squeeze::{self, SqueezeReport},
    stats::Pricing,
    truth::{self, StrictAnswer},
};

#[derive(Debug, Clone)]
pub struct GroundingOptions {
    pub budget_tokens: u64,
    pub limit: usize,
    pub answer: Option<String>,
    pub raw_tokens: Option<u64>,
    pub sent_tokens: Option<u64>,
    pub input_usd_per_million: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingReport {
    pub schema: String,
    pub query: String,
    pub local_only: bool,
    pub provider_calls: u64,
    pub hallucination_gate_passed: bool,
    pub verdict: String,
    pub risk_level: String,
    pub indexed_tokens: u64,
    pub used_tokens: u64,
    pub omitted_tokens: u64,
    pub retrieval_plan: RetrievalPlan,
    pub proof_per_token: ProofPerToken,
    pub strict_answer: StrictAnswer,
    pub answer_judgement: Option<JudgeReport>,
    pub savings_simulation: SavingsSimulation,
    pub prompt_contract: String,
    pub claim_policy: String,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalPlan {
    pub mode: String,
    pub budget_tokens: u64,
    pub indexed_tokens: u64,
    pub used_tokens: u64,
    pub omitted_tokens: u64,
    pub context_reduction_x: f64,
    pub final_route: String,
    pub stages: Vec<RetrievalStage>,
    pub adaptive_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalStage {
    pub name: String,
    pub status: String,
    pub used_tokens: u64,
    pub evidence_items: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPerToken {
    pub support_rate: f64,
    pub supported_claims: usize,
    pub partial_claims: usize,
    pub unsupported_claims: usize,
    pub evidence_items: usize,
    pub cited_quarks: usize,
    pub used_tokens: u64,
    pub proof_density: f64,
    pub tokens_per_supported_claim: f64,
    pub metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsSimulation {
    pub raw_input_tokens: u64,
    pub sent_input_tokens: u64,
    pub omitted_input_tokens: u64,
    pub reduction_x: f64,
    pub avoidance_rate_percent: f64,
    pub input_usd_per_million_tokens: f64,
    pub raw_input_cost_usd: f64,
    pub compact_input_cost_usd: f64,
    pub estimated_avoided_input_cost_usd: f64,
    pub pricing_source: String,
    pub boundary: String,
}

pub fn grounding_gate(
    index: &RepoIndex,
    query: &str,
    options: GroundingOptions,
) -> GroundingReport {
    let budget_tokens = options.budget_tokens.clamp(128, 20_000);
    let limit = options.limit.clamp(1, 16);

    let strict = truth::strict_answer(index, query, limit.min(8));
    let squeeze_budget = budget_tokens.clamp(128, 2_400);
    let squeezed = squeeze::squeeze_context(index, query, squeeze_budget, limit);
    let b2c = b2c_quant::plan_context(index, query, budget_tokens);
    let answer_judgement = options
        .answer
        .as_deref()
        .filter(|answer| !answer.trim().is_empty())
        .map(|answer| judge::judge_answer(index, answer, None));

    let retrieval_plan = retrieval_plan(budget_tokens, &strict, &squeezed, &b2c);
    let proof_per_token = proof_per_token(&strict, &squeezed, answer_judgement.as_ref());
    let hallucination_gate_passed = hallucination_gate_passed(&strict, answer_judgement.as_ref());
    let verdict = verdict(
        &strict,
        answer_judgement.as_ref(),
        hallucination_gate_passed,
    );
    let risk_level = risk_level(&verdict, &proof_per_token);
    let savings_simulation = simulate_savings(
        options.raw_tokens.unwrap_or(retrieval_plan.indexed_tokens),
        options.sent_tokens.unwrap_or(retrieval_plan.used_tokens),
        options.input_usd_per_million,
    );

    GroundingReport {
        schema: "qorx.grounding-gate.v1".to_string(),
        query: query.to_string(),
        local_only: true,
        provider_calls: 0,
        hallucination_gate_passed,
        verdict,
        risk_level,
        indexed_tokens: retrieval_plan.indexed_tokens,
        used_tokens: retrieval_plan.used_tokens,
        omitted_tokens: retrieval_plan.omitted_tokens,
        retrieval_plan,
        proof_per_token,
        strict_answer: strict,
        answer_judgement,
        savings_simulation,
        prompt_contract: "Use only cited Qorx evidence. If the gate is not grounded, refuse or ask Qorx to expand; do not fill gaps from model memory.".to_string(),
        claim_policy: "Qorx may claim estimated avoided input cost from local counters; no 100 percent hallucination claim and no provider invoice savings claim without routed provider evidence.".to_string(),
        boundary: "Grounding Gate is deterministic local evidence control: strict extract, squeeze, B2C planning, optional answer judging, and what-if input-cost math. It reduces hallucination risk by refusing unsupported claims; it cannot guarantee arbitrary downstream model truth.".to_string(),
    }
}

fn retrieval_plan(
    budget_tokens: u64,
    strict: &StrictAnswer,
    squeezed: &SqueezeReport,
    b2c: &B2cPlan,
) -> RetrievalPlan {
    let used_tokens = [strict.used_tokens, squeezed.used_tokens, b2c.used_tokens]
        .into_iter()
        .min()
        .unwrap_or(0)
        .max(strict.used_tokens);
    let indexed_tokens = strict
        .indexed_tokens
        .max(squeezed.indexed_tokens)
        .max(b2c.indexed_tokens);
    let omitted_tokens = indexed_tokens.saturating_sub(used_tokens.min(indexed_tokens));
    let context_reduction_x = indexed_tokens.max(1) as f64 / used_tokens.max(1) as f64;
    let final_route = if strict.coverage == "supported" {
        "strict-answer"
    } else if !squeezed.evidence.is_empty() {
        "squeeze"
    } else {
        b2c.route.as_str()
    }
    .to_string();

    RetrievalPlan {
        mode: "adaptive_grounded_retrieval".to_string(),
        budget_tokens,
        indexed_tokens,
        used_tokens,
        omitted_tokens,
        context_reduction_x: round2(context_reduction_x),
        final_route,
        stages: vec![
            RetrievalStage {
                name: "strict-answer".to_string(),
                status: strict.coverage.clone(),
                used_tokens: strict.used_tokens,
                evidence_items: strict.evidence.len(),
                reason: "first pass: exact extractive evidence or refusal".to_string(),
            },
            RetrievalStage {
                name: "squeeze".to_string(),
                status: if squeezed.evidence.is_empty() {
                    "empty"
                } else {
                    "evidence"
                }
                .to_string(),
                used_tokens: squeezed.used_tokens,
                evidence_items: squeezed.evidence.len(),
                reason: "second pass: query-relevant lines under a bounded budget".to_string(),
            },
            RetrievalStage {
                name: "b2c-plan".to_string(),
                status: b2c.route.clone(),
                used_tokens: b2c.used_tokens,
                evidence_items: b2c.selected_quarks.len(),
                reason: "third pass: budgeted quark portfolio with risk and redundancy penalties"
                    .to_string(),
            },
        ],
        adaptive_policy:
            "start extractive, expand only to cited squeeze/B2C evidence when support is incomplete"
                .to_string(),
    }
}

fn proof_per_token(
    strict: &StrictAnswer,
    squeezed: &SqueezeReport,
    judgement: Option<&JudgeReport>,
) -> ProofPerToken {
    let evidence_items = strict.evidence.len().max(squeezed.evidence.len());
    let cited_quarks = strict.evidence.len() + squeezed.evidence.len();
    let used_tokens = strict.used_tokens.max(1);
    let (supported_claims, partial_claims, unsupported_claims, total_claims) =
        if let Some(judgement) = judgement {
            let total = judgement.claims.len().max(1);
            (
                judgement.supported_claims,
                judgement.partial_claims,
                judgement.unsupported_claims,
                total,
            )
        } else {
            let supported = usize::from(strict.coverage == "supported");
            let partial = usize::from(strict.coverage == "partial");
            let unsupported = usize::from(strict.coverage == "not_found");
            (supported, partial, unsupported, 1)
        };
    let support_score = supported_claims as f64 + (partial_claims as f64 * 0.5);
    let support_rate = support_score / total_claims as f64;
    let tokens_per_supported_claim = if supported_claims == 0 {
        used_tokens as f64
    } else {
        used_tokens as f64 / supported_claims as f64
    };

    ProofPerToken {
        support_rate: round4(support_rate),
        supported_claims,
        partial_claims,
        unsupported_claims,
        evidence_items,
        cited_quarks,
        used_tokens,
        proof_density: round6(evidence_items as f64 / used_tokens as f64),
        tokens_per_supported_claim: round2(tokens_per_supported_claim),
        metric: "supported_claims_per_model_visible_token_with_cited_quark_evidence".to_string(),
    }
}

fn hallucination_gate_passed(strict: &StrictAnswer, judgement: Option<&JudgeReport>) -> bool {
    if let Some(judgement) = judgement {
        return judgement.unsupported_claims == 0
            && judgement.partial_claims == 0
            && judgement.supported_claims > 0;
    }
    strict.coverage == "supported" && !strict.evidence.is_empty()
}

fn verdict(
    strict: &StrictAnswer,
    judgement: Option<&JudgeReport>,
    hallucination_gate_passed: bool,
) -> String {
    if hallucination_gate_passed {
        return "grounded".to_string();
    }
    if let Some(judgement) = judgement {
        if judgement.unsupported_claims > 0 {
            return "blocked_unsupported_claims".to_string();
        }
        if judgement.partial_claims > 0 {
            return "partial_support_expand".to_string();
        }
    }
    if strict.evidence.is_empty() {
        "needs_more_evidence".to_string()
    } else {
        "partial_support_expand".to_string()
    }
}

fn risk_level(verdict: &str, proof: &ProofPerToken) -> String {
    match verdict {
        "grounded" if proof.support_rate >= 1.0 => "low".to_string(),
        "partial_support_expand" => "medium".to_string(),
        _ => "high".to_string(),
    }
}

fn simulate_savings(
    raw_input_tokens: u64,
    sent_input_tokens: u64,
    input_usd_per_million: Option<f64>,
) -> SavingsSimulation {
    let pricing = Pricing::from_env();
    let input_price = input_usd_per_million
        .filter(|value| *value >= 0.0)
        .unwrap_or(pricing.input_usd_per_million_tokens);
    let sent_input_tokens = sent_input_tokens.max(1);
    let omitted_input_tokens = raw_input_tokens.saturating_sub(sent_input_tokens);
    let raw_input_cost_usd = usd(raw_input_tokens, input_price);
    let compact_input_cost_usd = usd(sent_input_tokens, input_price);
    let estimated_avoided_input_cost_usd = (raw_input_cost_usd - compact_input_cost_usd).max(0.0);

    SavingsSimulation {
        raw_input_tokens,
        sent_input_tokens,
        omitted_input_tokens,
        reduction_x: round2(raw_input_tokens.max(1) as f64 / sent_input_tokens as f64),
        avoidance_rate_percent: round4(
            omitted_input_tokens as f64 / raw_input_tokens.max(1) as f64 * 100.0,
        ),
        input_usd_per_million_tokens: input_price,
        raw_input_cost_usd,
        compact_input_cost_usd,
        estimated_avoided_input_cost_usd: round6(estimated_avoided_input_cost_usd),
        pricing_source: if input_usd_per_million.is_some() {
            "cli_override".to_string()
        } else {
            pricing.source
        },
        boundary: "This is a what-if estimate for avoided input tokens. It is not a provider invoice and is valid only when the omitted context stays local and the sent evidence is sufficient.".to_string(),
    }
}

fn usd(tokens: u64, usd_per_million: f64) -> f64 {
    round6(tokens as f64 / 1_000_000.0 * usd_per_million)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
