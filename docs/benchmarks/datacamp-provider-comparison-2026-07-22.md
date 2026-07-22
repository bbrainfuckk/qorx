# DataCamp provider comparison — 2026-07-22

This is a small, matched functional check of full-context prompting versus a
Qorx 1.0.6 evidence pack. It is not a general model leaderboard, a cost
guarantee, or a rerun of the AMD benchmark.

## Result

| DataCamp sandbox | Model accepted by the live proxy | Full input tokens | Qorx input tokens | Reduction | Full accuracy | Qorx accuracy | Median Qorx pack |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Python with OpenAI | `gpt-4o-mini` | 65,850 | 814 | **80.90x** | 3/3 | 3/3 | 11.82 ms |
| Python with Anthropic | `claude-sonnet-4-6` | 77,458 | 963 | **80.43x** | 3/3 | 3/3 | 14.03 ms |

Both models answered the two supported questions correctly and returned the
exact refusal token `NOT_SUPPORTED` for the unsupported question under both
conditions.

## Fixed method

- Environment: DataCamp's managed Python with OpenAI and Python with Anthropic
  sandboxes.
- Qorx: `qorx 1.0.6`, official `linux-x64-static` release asset.
- Corpus: 106,758 characters, 26,690 local `ceil(chars / 4)` estimated tokens,
  and 26,645 tokens reported by the Qorx index.
- Tasks: two exact-answer questions and one deliberately unsupported question.
- Qorx budget: 320 tokens per question.
- Order: full context first, then Qorx context, for each question.
- Provider generation: temperature 0 and at most 80 output tokens.
- Scoring: deterministic expected-substring checks for supported answers and
  exact-match refusal for the unsupported answer.
- Repetitions: one call per question and condition. There are no confidence
  intervals.

The corpus, prompt contract, expected answers, per-call usage, answers, cache
fields, timings, and hashes are in the
[normalized raw capture](datacamp-provider-comparison-2026-07-22.json). The
[notebook](../../notebooks/Qorx_1_0_6_DataCamp_Providers.ipynb) reproduces the
fixture without embedding an API key.

## Artifact identity and portability finding

The release archive SHA-256 was
`c22dd19e666d7a12b19f80c9ff78922fe22abe6d423c45a3f32befe41d1c41ef`.
The extracted Qorx binary was 5,732,720 bytes with SHA-256
`1cbee5289a75610cbf8c812f4ce5626435712f1aae2e684a5d90565a7ac728c2`.

The first run exposed a real compatibility defect: the normal GNU Linux asset
required glibc symbols newer than DataCamp's glibc 2.27 image. Qorx's release
workflow now also publishes a static musl x64 asset, which was the binary used
for the results above.

The Anthropic starter template also named `claude-sonnet-4-0`, which the live
DataCamp proxy rejected. Its error listed `claude-haiku-4-5`,
`claude-sonnet-4-5`, and `claude-sonnet-4-6` as supported; the run used
`claude-sonnet-4-6`. This is a dated observation of the DataCamp proxy, not a
claim about Anthropic's full public model catalog.

## How to read the numbers

“Provider input tokens” is the usage field returned by each provider adapter.
It includes the entire input-token count reported for the request. In the
OpenAI run, the second and third full-context calls each reported 21,888 cached
input tokens. Therefore:

- the 80.90x OpenAI figure is a total reported-input ratio, not an uncached
  billing-savings ratio;
- sequential latency was affected by provider caching, network conditions, and
  request order;
- the provider latency figures should not be treated as a controlled speed
  benchmark;
- 3/3 accuracy is only a fixture result and does not support “zero
  hallucination” or general accuracy claims; and
- the roughly 80x results do not validate the separate 13.2M AMD carrier ratio,
  which has a different corpus and measurement boundary.

The useful conclusion is narrower: on this disclosed fixture, Qorx selected
137, 131, and 19 local tokens for the three questions; both provider models
preserved the tested answers and refusal while receiving much less input.

## Provider interfaces

The check used DataCamp's managed adapters for the
[OpenAI API](https://developers.openai.com/api/docs/overview) and
[Anthropic Messages API](https://docs.anthropic.com/en/api/messages). No API
key, DataCamp token, or private Qorx data is stored in this repository.
The OpenAI sandbox's supplied interface was Chat Completions, so this dated
capture should not be presented as a Responses API benchmark.
