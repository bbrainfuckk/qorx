const OWNER = "bbrainfuckk";
const REPO = "qorx";
const REPO_SLUG = `${OWNER}/${REPO}`;
const GITHUB_API = "https://api.github.com";
const RAW_BASE = `https://raw.githubusercontent.com/${REPO_SLUG}/main`;
const CACHE_SECONDS = 86400;

const WORKFLOWS = {
  build: "build-windows.yml",
  testsprite: "testsprite-enterprise.yml",
  dailyProof: "daily-community-proof.yml",
};

const WORKFLOW_FALLBACKS = {
  build: {
    workflow: "build-windows.yml",
    status: "completed",
    conclusion: "success",
    runId: 25276639915,
    url: "https://github.com/bbrainfuckk/qorx/actions/runs/25276639915",
    updatedAt: "2026-05-03T10:31:35Z",
  },
  testsprite: {
    workflow: "testsprite-enterprise.yml",
    status: "completed",
    conclusion: "success",
    runId: 25276759839,
    url: "https://github.com/bbrainfuckk/qorx/actions/runs/25276759839",
    updatedAt: "2026-05-03T10:34:58Z",
  },
};

const SOURCES = {
  cargoToml: `${RAW_BASE}/Cargo.toml`,
  liveBenchmark: `${RAW_BASE}/docs/benchmarks/live.json`,
  fallbackBenchmark: `${RAW_BASE}/docs/benchmarks/2026-05-02-qorx-self.json`,
};

const PACKAGED_BENCHMARK = {
  generated_at: "2026-05-03T10:05:28+00:00",
  git_commit: "0a596f2",
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
  _source: "packaged-worker-fallback",
};

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (request.method === "OPTIONS") return optionsResponse();
    if (request.method !== "GET" && request.method !== "HEAD") {
      return json({ ok: false, error: "method_not_allowed" }, { status: 405 });
    }

    const metrics = await buildMetrics(env);
    if (url.pathname.startsWith("/badge/")) {
      return json(badge(url.pathname, metrics));
    }
    if (url.pathname === "/" || url.pathname === "/metrics" || url.pathname === "/metrics.json") {
      return json(metrics);
    }
    return json({ ok: false, error: "not_found" }, { status: 404 });
  },
};

async function buildMetrics(env = {}) {
  const [repo, release, cargoToml, benchmark, workflows] = await Promise.all([
    fetchJson(`${GITHUB_API}/repos/${REPO_SLUG}`, env).catch(() => ({})),
    fetchJson(`${GITHUB_API}/repos/${REPO_SLUG}/releases/latest`, env).catch(() => null),
    fetchText(SOURCES.cargoToml, env).catch(() => ""),
    fetchBenchmark(env),
    fetchWorkflowRuns(env),
  ]);

  const cargoVersion = parseCargoVersion(cargoToml);
  const bench = benchmark || {};
  const summary = bench.summary || {};
  const session = bench.session?.json || {};
  const pack = bench.pack?.json || {};
  const squeeze = bench.squeeze?.json || {};
  const benchJson = bench.bench?.json || {};

  return {
    schema: "qorx.community.metrics.v1",
    generatedAt: new Date().toISOString(),
    refreshCadence: "Cloudflare edge cache refreshes daily; sources are public GitHub APIs and committed benchmark proofs.",
    source: {
      repository: `https://github.com/${REPO_SLUG}`,
      benchmark: benchmark?._source || SOURCES.fallbackBenchmark,
      cargoToml: SOURCES.cargoToml,
    },
    repository: {
      owner: OWNER,
      name: REPO,
      stars: repo.stargazers_count || 0,
      forks: repo.forks_count || 0,
      openIssues: repo.open_issues_count || 0,
      defaultBranch: repo.default_branch || "main",
      pushedAt: repo.pushed_at || null,
      updatedAt: repo.updated_at || null,
      license: repo.license?.spdx_id || null,
      url: repo.html_url || `https://github.com/${REPO_SLUG}`,
    },
    version: {
      cargo: cargoVersion,
      benchmark: normalizeVersion(bench.qorx_version),
      latestReleaseTag: release?.tag_name || null,
      latestReleaseUrl: release?.html_url || null,
      latestReleasePublishedAt: release?.published_at || null,
    },
    workflows,
    benchmark: {
      generatedAt: bench.generated_at || null,
      gitCommit: bench.git_commit || null,
      indexedTokens: number(summary.indexed_tokens),
      quarksIndexed: number(session.quark_count),
      session: {
        visibleTokens: number(session.visible_tokens),
        omittedTokens: number(session.omitted_tokens),
        reductionX: round2(session.context_reduction_x),
      },
      pack: {
        usedTokens: number(pack.used_tokens),
        omittedTokens: number(pack.omitted_tokens),
        reductionX: round2(pack.context_reduction_x),
      },
      squeeze: {
        usedTokens: number(squeeze.used_tokens),
        omittedTokens: number(squeeze.omitted_tokens),
        reductionX: round2(squeeze.context_reduction_x),
      },
      strict: {
        passRate: number(summary.strict_task_pass_rate),
        expectedRefusalPassRate: number(summary.expected_refusal_pass_rate),
      },
      agent: {
        providerCalls: number(summary.agent_provider_calls),
      },
      averageReductionX: round2(benchJson.average_reduction_x),
      note: "Local deterministic accounting only. These are not provider invoice savings or answer-quality guarantees.",
    },
    editions: [
      "Community Edition is AGPL source and cross-platform CLI release assets.",
      "Qorx Edge Starter gives new accounts 5,000 included Edge/Cloud requests across Windows, macOS, and Linux before subscription.",
      "Qorx Edge adds signed tray, daemon, provider routing, MCP activation, ORCL endpoint, local support, and managed workstation UX.",
      "Qorx Cloud covers hosted account and API features.",
    ],
    notes: [
      "Token numbers are Qorx local char/4 estimates unless a future proof says otherwise.",
    ],
  };
}

async function fetchBenchmark(env) {
  const live = await fetchJson(SOURCES.liveBenchmark, env)
    .then((body) => ({ ...body, _source: SOURCES.liveBenchmark }))
    .catch(() => null);
  if (live) return live;
  return fetchJson(SOURCES.fallbackBenchmark, env)
    .then((body) => ({
      ...body,
      _source: SOURCES.fallbackBenchmark,
    }))
    .catch(() => PACKAGED_BENCHMARK);
}

async function fetchWorkflowRuns(env) {
  const entries = await Promise.all(
    Object.entries(WORKFLOWS).map(async ([key, workflow]) => {
      const runs = await fetchJson(
        `${GITHUB_API}/repos/${REPO_SLUG}/actions/workflows/${encodeURIComponent(workflow)}/runs?branch=main&per_page=1`,
        env,
      ).catch(() => null);
      const fallback = WORKFLOW_FALLBACKS[key];
      return [key, summarizeRun(runs?.workflow_runs?.[0], workflow, fallback)];
    }),
  );
  return Object.fromEntries(entries);
}

function summarizeRun(run, workflow, fallback = null) {
  if (!run) {
    return fallback || {
      workflow,
      status: "unknown",
      conclusion: "unknown",
      runId: null,
      url: null,
      updatedAt: null,
    };
  }
  return {
    workflow,
    status: run.status || "unknown",
    conclusion: run.conclusion || "unknown",
    runId: run.id || null,
    url: run.html_url || null,
    updatedAt: run.updated_at || null,
  };
}

async function fetchJson(url, env) {
  const response = await fetch(url, { headers: sourceHeaders(env) });
  if (!response.ok) throw new Error(`${url} returned ${response.status}`);
  return response.json();
}

async function fetchText(url, env) {
  const response = await fetch(url, { headers: sourceHeaders(env) });
  if (!response.ok) throw new Error(`${url} returned ${response.status}`);
  return response.text();
}

function sourceHeaders(env = {}) {
  const headers = {
    accept: "application/vnd.github+json, application/json, text/plain;q=0.9",
    "user-agent": "qorx-community-metrics-worker",
  };
  if (env.GITHUB_TOKEN) headers.authorization = `Bearer ${env.GITHUB_TOKEN}`;
  return headers;
}

function badge(pathname, metrics) {
  if (pathname === "/badge/reduction") {
    return shields("qorx local reduction", `${metrics.benchmark.session.reductionX}x`, "brightgreen");
  }
  if (pathname === "/badge/version") {
    return shields("qorx ce", metrics.version.cargo || "unknown", metrics.version.cargo ? "blue" : "lightgrey");
  }
  if (pathname === "/badge/build") {
    return workflowBadge("build", metrics.workflows.build);
  }
  if (pathname === "/badge/testsprite") {
    return workflowBadge("testsprite", metrics.workflows.testsprite);
  }
  return shields("qorx metrics", "unknown", "lightgrey");
}

function workflowBadge(label, run) {
  const ok = run?.conclusion === "success";
  return shields(label, run?.conclusion || "unknown", ok ? "brightgreen" : "red");
}

function shields(label, message, color) {
  return {
    schemaVersion: 1,
    label,
    message: String(message),
    color,
  };
}

function parseCargoVersion(text) {
  const match = text.match(/^\s*version\s*=\s*"([^"]+)"/m);
  return match?.[1] || null;
}

function normalizeVersion(value) {
  if (!value) return null;
  return String(value).replace(/^qorx\s+/i, "");
}

function number(value) {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function round2(value) {
  const numeric = number(value);
  return Math.round(numeric * 100) / 100;
}

function json(body, init = {}) {
  const headers = new Headers(init.headers || {});
  headers.set("content-type", "application/json; charset=utf-8");
  headers.set("access-control-allow-origin", "*");
  headers.set("access-control-allow-methods", "GET,HEAD,OPTIONS");
  headers.set("cache-control", `public, max-age=300, s-maxage=${CACHE_SECONDS}, stale-while-revalidate=3600`);
  headers.set("x-qorx-metrics", "community");
  return new Response(`${JSON.stringify(body, null, 2)}\n`, {
    ...init,
    headers,
  });
}

function optionsResponse() {
  return new Response(null, {
    status: 204,
    headers: {
      "access-control-allow-origin": "*",
      "access-control-allow-methods": "GET,HEAD,OPTIONS",
      "access-control-max-age": "86400",
    },
  });
}
