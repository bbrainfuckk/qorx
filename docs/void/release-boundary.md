# Release Boundary

Qorx has two public-facing surfaces.

## Main Public Repository

The main public repository is allowed to contain:

- documentation;
- benchmark summaries;
- research references;
- citation metadata;
- license, notice, trademark, and security files;
- GitHub Release assets for approved public packages.

It should not contain source, unpublished technical material, private assets, or release archives committed into git history.

## GitHub Release Assets

`qorx-free` may be uploaded as a GitHub Release asset because it is a bounded benchmarker.

Safe release asset:

```text
qorx-free-0.0.1-linux-amd-mi300x-x64.tar.gz
```

The asset should be attached to a tag such as:

```text
qorx-free-v0.0.1
```

The release should say:

> `qorx-free` is a public Linux AMD MI300X benchmark and reproducibility build. It verifies the public bundle, checks the hardware boundary, and writes sanitized benchmark artifacts. It is not Qorx Void and does not include private product material.

## Void Distribution

Qorx Void can be documented in this public repository.

Qorx Void should not be published as a public release asset unless the shipped package has a separate release audit proving that it contains only approved public material.

Until then:

- public repo documents Void;
- `qorx-free` serves public benchmarkers;
- full Qorx Void remains a private product/runtime distribution.
