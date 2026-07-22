import assert from "node:assert/strict";
import test from "node:test";

import worker from "../cloudflare/community-metrics-worker.mjs";

const repoPayload = {
  stargazers_count: 42,
  forks_count: 7,
  open_issues_count: 3,
  default_branch: "main",
  pushed_at: "2026-05-03T00:00:00Z",
  updated_at: "2026-05-03T00:00:00Z",
  html_url: "https://github.com/bbrainfuckk/qorx",
  license: { spdx_id: "AGPL-3.0-only" },
};

const releasePayload = {
  tag_name: "v0.0.1-ylem",
  html_url: "https://github.com/bbrainfuckk/qorx/releases/tag/v0.0.1-ylem",
  published_at: "2026-05-03T00:00:00Z",
};

const workflowPayload = {
  workflow_runs: [
    {
      id: 25274157669,
      conclusion: "success",
      status: "completed",
      html_url: "https://github.com/bbrainfuckk/qorx/actions/runs/25274157669",
      updated_at: "2026-05-03T08:25:10Z",
    },
  ],
};

const benchmarkPayload = {
  generated_at: "2026-05-10T01:50:01+00:00",
  git_commit: "34173a6",
  qorx_version: "qorx 0.0.1-ylem",
  summary: {
    indexed_tokens: 388573,
    strict_task_pass_rate: 1,
    expected_refusal_pass_rate: 1,
    agent_provider_calls: 0,
  },
  session: {
    json: {
      quark_count: 710,
      visible_tokens: 69,
      omitted_tokens: 388504,
      context_reduction_x: 5631.492753623188,
    },
  },
  pack: {
    json: {
      used_tokens: 410,
      omitted_tokens: 388163,
      context_reduction_x: 947.7390243902439,
    },
  },
  squeeze: {
    json: {
      used_tokens: 448,
      omitted_tokens: 388125,
      context_reduction_x: 867.3504464285714,
    },
  },
  bench: {
    json: {
      average_reduction_x: 877.1377703830394,
    },
  },
};

function mockFetch() {
  return async (url) => {
    const href = String(url);
    if (href.endsWith("/repos/bbrainfuckk/qorx")) return jsonResponse(repoPayload);
    if (href.endsWith("/repos/bbrainfuckk/qorx/releases/latest")) return jsonResponse(releasePayload);
    if (href.includes("/actions/workflows/")) return jsonResponse(workflowPayload);
    if (href.includes("/Cargo.toml")) return new Response('version = "0.0.1-ylem"\n');
    if (href.includes("/docs/benchmarks/live.json")) return jsonResponse(benchmarkPayload);
    throw new Error(`unexpected fetch ${href}`);
  };
}

function jsonResponse(body) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
  });
}

test("community metrics worker returns live proof numbers", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = mockFetch();

  try {
    const response = await worker.fetch(new Request("https://metrics.example.test/"));
    assert.equal(response.status, 200);
    assert.equal(response.headers.get("access-control-allow-origin"), "*");
    assert.match(response.headers.get("cache-control"), /s-maxage=86400/);

    const body = await response.json();
    assert.equal(body.schema, "qorx.community.metrics.v1");
    assert.equal(body.version.cargo, "0.0.1-ylem");
    assert.equal(body.repository.stars, 42);
    assert.equal(body.benchmark.session.reductionX, 5631.49);
    assert.equal(body.benchmark.strict.passRate, 1);
    assert.equal(body.benchmark.agent.providerCalls, 0);
    assert.match(body.editions.join(" "), /Qorx Void/);
    assert.match(body.editions.join(" "), /Qorx Void Starter/);
    assert.match(body.editions.join(" "), /5,000 included Void\/Cloud requests/);
    assert.doesNotMatch(JSON.stringify(body), /Qorx Local Pro|boundary/i);
    assert.equal(body.workflows.build.conclusion, "success");
    assert.equal(body.workflows.testsprite.conclusion, "success");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("community metrics worker returns shields badges", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = mockFetch();

  try {
    const response = await worker.fetch(new Request("https://metrics.example.test/badge/reduction"));
    assert.equal(response.status, 200);
    const body = await response.json();
    assert.equal(body.schemaVersion, 1);
    assert.equal(body.label, "qorx local reduction");
    assert.match(body.message, /5631\.49x/);
    assert.equal(body.color, "brightgreen");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("community metrics worker keeps badges alive when upstream fetches fail", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => new Response("temporarily unavailable", { status: 503 });

  try {
    const response = await worker.fetch(new Request("https://metrics.example.test/badge/reduction"));
    assert.equal(response.status, 200);
    const body = await response.json();
    assert.equal(body.schemaVersion, 1);
    assert.equal(body.message, "5631.49x");
    assert.equal(body.color, "brightgreen");
  } finally {
    globalThis.fetch = originalFetch;
  }
});
