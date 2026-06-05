# Day-To-Day Use

Qorx Void and `qorx-free` serve different users.

## Qorx Void Users

Qorx Void is for people who work with Codex on real projects and do not want to repeat the same context every turn.

Day to day, the user should experience:

- less repeated project explanation;
- fewer manual context restarts;
- compact evidence when Codex needs local support;
- local-first handling of workspace memory;
- clear refusal when a claim is not supported by local evidence.

Public docs can describe this experience. They should not publish the private product material behind it.

## `qorx-free` Testers

`qorx-free` is for public testers with Linux AMD MI300X machines.

Useful daily commands:

```sh
./qorx-free hardware
./qorx-free doctor
./qorx-free verify-demo
./qorx-free amd-run --suite big10 --sample 30 --distractors 12 --out ./qorx-free-run
```

Testers can use it after:

- ROCm updates;
- kernel updates;
- model endpoint changes;
- machine rebuilds;
- reproducibility checks;
- benchmark issue reports.

## What To Attach To Issues

When reporting a public `qorx-free` benchmark issue, attach only sanitized outputs:

- `qorx-free doctor --json` output;
- `qorx-free-amd-run.json`;
- `qorx-free-amd-run-manifest.json`;
- `qorx-free-amd-run-checksums.txt`;
- hardware and ROCm version notes.

Do not attach:

- private repository files;
- raw prompts;
- model response bodies;
- secret values;
- local usernames;
- private hostnames;
- unpublished technical material;
- private Qorx logs.
