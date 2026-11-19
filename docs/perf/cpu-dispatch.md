# CPU dispatch and AVX-512

`mw-cpu` implements an actual AVX-512F + AVX-512BW kernel for ASCII
lowercasing. It loads 64 bytes at a time, compares unsigned bytes against `A`
and `Z`, then adds 32 only under the matching byte mask. Unaligned loads and
stores stay inside complete 64-byte chunks; the remaining bytes use the standard
library. Bytes outside ASCII uppercase remain unchanged, preserving UTF-8.
The current production caller is scanner-path normalization in `mw-protection`.

## Runtime selection

The portable server binary checks CPU and operating-system support with Rust's
runtime feature detection before entering a kernel. AVX-512 requires **both F
and BW**; F alone is insufficient. An AVX2 kernel processes 32-byte chunks where
AVX-512 is unavailable or the input is smaller. Other targets retain the portable
implementation. There are no global `target-cpu=native` flags, nightly features
or unconditional AVX-512 instructions in the calling code.

| Input length | Automatic selection |
| --- | --- |
| Under 256 bytes | Standard library directly, without feature discovery |
| 256–511 bytes | AVX2 when supported; otherwise portable |
| 512 bytes or more | AVX-512F + BW when supported; otherwise AVX2 or portable |

These thresholds are implementation choices, not a speed guarantee. Tiny inputs
avoid dispatch overhead. `lowercase_ascii_string` provides the safe string API;
all unsafe memory and instruction handling stays inside the audited leaf crate.
The explicit backend API exists for testing and benchmarking. An unsupported
backend returns an error before modifying the input; it is not an administrator
configuration that can force unsupported instructions.

## Resource reporting

Observability → Resources shows the architecture, selected lowercase kernel for
large inputs, available CPU features and dispatch thresholds. The feature list
reports availability, **not that the app uses every listed feature**. AES, SHA,
BMI and AVX-512VL detection does not imply those features are used by this kernel.
Short inputs can use a different path from the large-input kernel shown there.

## Verify and measure

Run without a competing build or CPU-heavy workload:

```sh
cargo test -p mw-cpu --lib
cargo test -p mw-cpu --release --lib lowercase_release_benchmark -- --ignored --nocapture --test-threads=1
cargo test -p mw-protection --release --lib request_probe_single_normalization_benchmark -- --ignored --nocapture --test-threads=1
```

Parity tests cover all byte values, offsets 0–65, lengths 0–512, tails, Unicode,
selection thresholds and rejection of unavailable backends. Hardware-specific
kernels run only when supported by the test host.

The CPU benchmark warms and checks each supported backend before measuring
32, 64, 128, 256, 512, 1,024, 2,048, 4,096 and 8,192-byte inputs. Seven samples rotate and alternate
backend order. It prints median time for automatic selection, the portable
implementation and supported forced kernels. Every measurement includes the
same input-reset copy; a separate copy-only median exposes that preparation cost.
These are kernel measurements, not end-to-end HTTP latency or build performance.

## Recorded measurements

The expanded release benchmark ran on 19 November 2026 with:

- Windows x86_64.
- Two Intel Xeon Gold 6138 processors at a nominal 2.00 GHz, each with 20 cores
  and 40 logical processors: 40 physical cores / 80 logical processors total.
- `rustc 1.98.1 (48a229cea 2026-09-01)`.
- Detected AVX-512F + BW support; the forced AVX-512 kernel actually executed.
- One benchmark test thread, seven samples per backend/length, rotated and
  alternating backend order. The harness does not control CPU affinity,
  frequency or thermal state.

The final release benchmark passed after applying the current **256-byte AVX2 /
512-byte AVX-512 thresholds**. The table records median nanoseconds per operation
from `.orchestration/reports/optimization-validation/cpu-release-final.log`.
Every backend includes the same input-reset copy; the copy-only column exposes
that preparation. No competing build ran during this measurement.

| Input bytes | Copy only | Automatic, current thresholds | Portable | Forced AVX2 | Forced AVX-512F + BW |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 3.37 | 9.14 | 6.98 | 12.66 | 15.66 |
| 64 | 6.86 | 13.36 | 12.71 | 13.96 | 17.82 |
| 128 | 7.34 | 18.36 | 19.24 | 18.11 | 18.63 |
| 256 | 8.27 | 24.62 | 25.17 | 22.30 | 20.51 |
| 512 | 12.04 | 31.04 | 45.67 | 32.73 | 31.37 |
| 1,024 | 19.65 | 54.16 | 85.67 | 55.48 | 56.29 |
| 2,048 | 31.27 | 66.82 | 149.46 | 84.69 | 73.59 |
| 4,096 | 80.62 | 171.38 | 321.69 | 192.02 | 177.92 |
| 8,192 | 130.75 | 349.33 | 700.74 | 366.36 | 344.36 |

The automatic path is close to the portable measurement for short inputs and
faster for the larger fixtures in this run. Forced backend rankings are not
identical at every size across runs: at 256 bytes the final forced AVX-512 sample
beat AVX2, reversing the earlier run, while the 1 KiB forced kernels remained
close. The 256/512-byte thresholds are conservative dispatch choices, not a claim
that one backend wins at every size or on every supported CPU.

The direct portable path and inlining annotations do not establish zero
short-call overhead: the measured automatic 32-byte call took 9.14 ns versus
6.98 ns for the explicit portable path, and the 64-byte automatic call took
13.36 ns versus 12.71 ns. Automatic and forced APIs include different checks and
call paths; compiler decisions, frequency, scheduling and benchmark structure
remain part of these numbers.

These results apply to this host, compiler, input distribution and harness.
They do not establish an application-wide speedup, improved Rust build time,
HTTP capacity across 80 logical processors, or equivalent results on other CPUs.

The final release test executable was also inspected with `objdump`: its vector
code contains ZMM loads, `vpcmpltub` mask comparisons, `vpaddb`, and masked
`vmovdqu8` stores. The instruction excerpt is retained in
`.orchestration/reports/optimization-validation/cpu-avx512-disassembly.txt`.
This supplements the runtime parity tests and confirms emitted AVX-512 code.
