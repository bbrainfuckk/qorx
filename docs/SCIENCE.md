# Qorx Science

Qorx is built around one measurable boundary:

```text
large local context -> small carrier -> cited proof on demand
```

The backend does not pretend that tokens disappear. It measures how much local
context was available, how much was sent, and which proof was selected.

## What Qorx Measures

| Surface | What it proves | What it does not prove |
| --- | --- | --- |
| Session carrier | A tiny `qorx://s/...` handle can point to indexed local state. | A remote model can understand hidden files without a resolver. |
| Evidence pack | Qorx can select cited local chunks under a budget. | The answer is automatically correct for every task. |
| Squeeze | Qorx can remove unrelated lines before sending context. | Neural compression quality. |
| Grounding gate | Claims can be checked against indexed evidence. | A universal zero-hallucination guarantee. |
| Cache plan | Stable and dynamic prompt regions can be separated. | Provider cache hits without upstream telemetry. |
| B2C accounting | Baseline-to-Compact local reduction can be estimated. | Final provider invoice savings. |

## Why It Works

Qorx treats context as local evidence, not one giant prompt.

1. Index local files into small bounded evidence chunks.
2. Store path, line, symbol, sparse-term, and token-estimate signals.
3. Rank evidence for the current objective.
4. Pack only the useful slices under a declared budget.
5. Refuse when indexed evidence cannot support the answer.

The research direction is conservative: prompt compression, graph-aware code
retrieval, cache-aware request layout, and grounded generation are useful only
when the implementation can show what was selected and what was omitted.

## Live Proof Commands

Run these from the repository root:

```powershell
cargo test --locked
cargo run --locked -- --version
cargo run --locked -- science
cargo run --locked -- adapters
cargo run --locked -- index .
cargo run --locked -- bench --budget-tokens 900 "resolver boundary proof"
cargo run --locked -- strict-answer "which files explain the resolver boundary?"
```

The expected version line for this repo is:

```text
qorx 1.0.6
```

## Claim Boundary

Use Qorx numbers as local evidence:

- indexed local tokens
- visible or sent tokens
- omitted local context
- local reduction ratio
- proof pages selected
- provider cached tokens only when provider metadata exposes them

Do not describe these as guaranteed provider invoice savings. Provider bills
depend on model, output tokens, account pricing, cache rules, and the actual
routed request.

## Reader Path

If you are evaluating Qorx, start with the trial that matches your work:

- Qorx Void Desktop: free 1-hour local demo.
- Qorx Cloud API: free 5,000 hosted calls.

Try it first. The counters are easier to trust after you watch them move.
