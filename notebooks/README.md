# Qorx notebooks

`Qorx_1_0_5_DataCamp_CPU.ipynb` is the canonical CPU verification notebook for
Qorx 1.0.5. It selects the matching GitHub release asset, verifies the reported
version, checks and compiles a Qorx program, executes the resulting bytecode, and
writes a machine-readable timing report.

The notebook accepts `QORX_BIN` when a binary is uploaded directly to DataCamp.
It does not contain a bundled executable or secret, and it does not relabel
measurements from an older Qorx build as 1.0.5 results.
