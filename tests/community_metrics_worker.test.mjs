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
  tag_name: "v1.0.4",
  html_url: "https://github.com/bbrainfuckk/qorx/releases/tag/v1.0.4",
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
  generated_at: "2026-05-03T00:00:00Z",
  git_commit: "abc1234",
  qorx_version: "qorx 1.0.4",
  summary: {
    indexed_tokens: 189042,
    strict_task_pass_rate: 1,
    expected_refusal_pass_rate: 1,
    agent_provider_calls: 0,
  },
  session: {
    json: {
      quark_count: 349,
      visible_tokens: 69,
      omitted_tokens: 188973,
      context_reduction_x: 2739.7391304347825,
    },
  },
  pack: {
    json: {
      used_tokens: 566,
      omitted_tokens: 188476,
      context_reduction_x: 333.9964664310954,
    },
  },
  squeeze: {
    json: {
      used_tokens: 292,
      omitted_tokens: 188750,
      context_reduction_x: 647.4041095890411,
    },
  },
  bench: {
    json: {
      average_reduction_x: 380.3280892831784,
    },
  },
};

function mockFetch() {
  return async (url) => {
    const href = String(url);
    if (href.endsWith("/repos/bbrainfuckk/qorx")) return jsonResponse(repoPayload);
    if (href.endsWith("/repos/bbrainfuckk/qorx/releases/latest")) return jsonResponse(releasePayload);
    if (href.includes("/actions/workflows/")) return jsonResponse(workflowPayload);
    if (href.includes("/Cargo.toml")) return new Response('version = "1.0.4"\n');
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
    assert.equal(body.version.cargo, "1.0.4");
    assert.equal(body.repository.stars, 42);
    assert.equal(body.benchmark.session.reductionX, 2739.74);
    assert.equal(body.benchmark.strict.passRate, 1);
    assert.equal(body.benchmark.agent.providerCalls, 0);
    assert.match(body.editions.join(" "), /Qorx Edge/);
    assert.match(body.editions.join(" "), /Qorx Edge Starter/);
    assert.match(body.editions.join(" "), /5,000 included Edge\/Cloud requests/);
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
    assert.match(body.message, /2739\.74x/);
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
    assert.equal(body.message, "2739.74x");
    assert.equal(body.color, "brightgreen");
  } finally {
    globalThis.fetch = originalFetch;
  }
});
