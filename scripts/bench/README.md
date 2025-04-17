# Bench Script

This folder contains the `bench.sh` script and a `results/` directory for collecting performance metrics and profiling data for RISC Zero zkVM guest executions.

## Features

- **Auto-detect host specs**: OS, architecture, CPU cores, and total RAM.
- **Per-host results**: Creates a subfolder under `results/` named `{OS}_{ARCH}_{CORES}c_{RAM}G`.
- **Performance CSV**: Appends detailed metrics to `bench_results.csv`, including:
  1. `label` – descriptive run identifier (function & params).
  2. `wall_s` – real (wall-clock) elapsed time.
  3. `user_s` – CPU time spent in user mode.
  4. `sys_s` – CPU time spent in kernel mode.
  5. `maxrss_kB` – peak resident set size.
  6. `cycles` – total RISC Zero guest cycles.
  7. `cpu_pct` – average CPU utilization `(user+sys)/wall*100`.
  8. `mem_pct` – memory usage as a percent of total RAM.
  9. `exec_time_ms` – zkVM reported execution time in milliseconds.
  10. `segments` – number of zkVM execution segments.
  11. `rz_total_cycles` – redundant with `cycles` (guest total cycles).
  12. `user_cycles` – guest user cycles.
  13. `user_pct` – guest user cycles percent.
  14. `paging_cycles` – guest paging cycles.
  15. `paging_pct` – guest paging cycles percent.
  16. `reserved_cycles` – guest reserved cycles.
  17. `reserved_pct` – guest reserved cycles percent.
  18. `timestamp` – UTC run timestamp.

- **Profile PB files**: Moves each `profile.pb` into the host-specific folder, renaming it to `profile_<fn>_<ts>.pb`.
- **Verbose debug**: On failure, prints full log of both host `time` and RISC Zero zkVM output.

## Prerequisites

- **Rust toolchain** with the `host` binary target.
- **RISC Zero** dependencies installed and available in your environment.
- **GNU `time`** on Linux (built-in) or macOS (install via `brew install gnu-time` for GNU features), or use the default `/usr/bin/time -l` on macOS.
- **`just`** command runner (optional).

## Usage with `just`

This folder is invoked by the `justfile` recipe:

```just
bench function params context_state program_spec value="0":
    @bash scripts/bench.sh \
      "{{function}}" \
      "{{params}}" \
      "{{context_state}}" \
      "{{program_spec}}" \
      "{{value}}"
```

Run the benchmark via:

```bash
just bench function=exploit(bool) \
           params=true \
           context_state=./ctx.json \
           program_spec=./spec.json \
           value=0
```

Results will appear in:

```
scripts/bench/results/{OS}_{ARCH}_{CORES}c_{RAM}G/bench_results.csv
scripts/bench/results/{OS}_{ARCH}_{CORES}c_{RAM}G/profile_<fn>_<ts>.pb
```

## Direct invocation

Without `just`, you can call the script directly:

```bash
bash scripts/bench.sh "exploit(bool)" "true" \
    ./ctx.json ./spec.json 0
```

## Visualizing Profiles

Use the Go `pprof` tool to visualize the `.pb` files:

```bash
go tool pprof -http=localhost:8000 results/.../profile_exploit_bool_*.pb
```

Navigate to the flame graph or top view in your browser.

## .gitignore

The `results/` directory is ignored by Git. To clear old results:

```bash
rm -rf scripts/bench/results/*
```

