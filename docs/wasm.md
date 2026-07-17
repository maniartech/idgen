# WebAssembly support

Findings and decisions for compiling idgen to WebAssembly. Written 2026-07-17,
against idgen 1.5.0.

Everything below was verified by building and running, not inferred. Reproduction
steps are at the bottom.

## Status

**Decided: WASI (`wasm32-wasip1`) is the target for 1.6.0.** It works today with
no code changes for 10 of the 11 ID types. A browser-native build
(`wasm32-unknown-unknown` + wasm-bindgen) is deferred; see
[Why not the browser target first](#why-not-the-browser-target-first).

Not yet implemented. Open questions are listed at the end.

## The two targets are not interchangeable

This is the thing to get straight first, because "compile to wasm" means two
quite different projects:

| | `wasm32-wasip1` (WASI) | `wasm32-unknown-unknown` (browser) |
|---|---|---|
| Host | wasmtime, wasmer, node's `node:wasi` | browsers, natively |
| Clock | provided by WASI | **absent — traps** |
| Randomness | provided by WASI | **absent — needs `getrandom`'s `js`** |
| Hostname | absent | absent |
| Runs in a browser | only via a JS WASI shim | yes |
| Interface | argv in, stdout out | real JS functions via wasm-bindgen |
| Works today | yes, 10/11 types, no changes | no |

WASI is the easier target *because* it has a clock and an RNG. The browser target
is where all the difficulty lives.

## The central constraint

On `wasm32-unknown-unknown`, `std::time::SystemTime::now()` **traps at runtime**.
It compiles cleanly, which is the trap — a successful `cargo build` proves
nothing here.

Verified with a minimal module run under node:

```
add(2,3):                          5              <- wasm itself works
clock_now():                       TRAPS          <- RuntimeError: unreachable
systemtime_from_millis(1784...):   round-trips    <- construction WORKS
```

The third line is the important one. You cannot *read* the clock in the browser
target, but you can *construct* a `SystemTime`:

```rust
let t = UNIX_EPOCH + Duration::from_millis(js_supplied_millis);
```

That arithmetic never touches the platform clock, and it round-trips exactly. So
any crate that accepts a caller-supplied timestamp can be made to work by
feeding it `Date.now()` from JS. Crates that only call `SystemTime::now()`
internally cannot.

That single distinction decides which ID types are salvageable on the browser
target.

## Per-type support

| Type | WASI | Browser | Notes |
|---|---|---|---|
| uuid3 / uuid5 | works | works today | pure hash: no clock, no RNG |
| uuid4 | works | needs `getrandom` `js` | RNG only |
| uuid1 / uuid6 / uuid7 | works | needs uuid's `js` feature | the crate has one, built for this |
| nanoid | works | needs `getrandom` `js` | RNG only |
| ulid | works | needs time injected | `Ulid::from_datetime(SystemTime)` is a usable seam |
| objectid | works | needs manual construction | `ObjectId::from_bytes([u8; 12])` |
| cuid2 | works | **hard** | no time-injection API |
| cuid1 | **panics** | **hard** | needs a hostname; see below |
| `inspect` | works | works today | pure parsing — chrono conversion is arithmetic, not a clock read |

Note the inversion: **cuid2 works under WASI but is the hardest type in the
browser.** Choosing WASI avoids the problem entirely.

`inspect` needs nothing on either target. The whole read-side of the tool is
already portable.

## cuid1 panics under WASI

```
thread 'main' panicked at library/std/src/sys/pal/wasi/os.rs:137:5:
unsupported
```

cuid1 derives part of its fingerprint from the system hostname, and WASI has no
hostname to give it. The panic comes from std, below the cuid crate, so there is
nothing to catch.

This is the one behaviour that must change before shipping. A panic reading
`unsupported` is a bad failure mode. Two options:

1. **Reject at the CLI layer** on wasm targets, with a real message explaining
   that cuid1 requires a hostname. Honest and cheap.
2. **Fall back to a random fingerprint.** Keeps the type available, but silently
   changes cuid1's collision characteristics on one platform — the same class of
   quiet-divergence problem as the old hardcoded v1 node ID.

Option 1 is preferred. Do not ship the panic.

## Why not the browser target first

The playground needs a browser build, so the browser target looks like the
obvious first move. It is not, for three reasons:

1. **WASI already works.** 10/11 types, zero code changes. The browser target
   needs a clock abstraction, two feature flags, and bespoke work for ulid and
   objectid — and still cannot do cuid.
2. **A WASI build can serve a browser playground** via a JS WASI shim
   (e.g. `@bjorn3/browser_wasi_shim`): construct argv, capture stdout. The
   playground then runs *the real CLI*, so what a visitor tries is exactly what
   they would install. No second code path to keep in sync.
3. **WASI is independently useful** beyond the playground — idgen becomes
   runnable under wasmtime/wasmer, in sandboxes, and in wasm plugin hosts.

The costs are real and should be stated: the artefact is **2.32 MB** (release;
38.94 MB debug), and the interface is a CLI rather than JS functions. A
wasm-bindgen build would be smaller and nicer to call. That is a later release,
if the shim proves limiting.

## Verified measurements

| | Size |
|---|---|
| `wasm32-wasip1` release | 2.32 MB |
| `wasm32-wasip1` debug | 38.94 MB |

2.32 MB is untested against `wasm-opt`, which would likely reduce it
meaningfully. Worth measuring before the playground depends on it.

## Reproducing

Prerequisites: `rustup target add wasm32-wasip1`, and node (for `node:wasi`).

```bash
cargo build --release --target wasm32-wasip1 --bin idgen
```

Run it with a small node harness:

```js
// wasi-run.mjs
import { WASI } from 'node:wasi';
import fs from 'node:fs';
const args = ['idgen', ...process.argv.slice(3)];
const wasi = new WASI({ version: 'preview1', args, env: {}, returnOnExit: true });
const wasm = await WebAssembly.compile(fs.readFileSync(process.argv[2]));
wasi.start(await WebAssembly.instantiate(wasm, wasi.getImportObject()));
```

```bash
node --no-warnings wasi-run.mjs target/wasm32-wasip1/release/idgen.wasm -t uuid7
node --no-warnings wasi-run.mjs target/wasm32-wasip1/release/idgen.wasm inspect 017F22E2-79B0-7CC3-98C4-DC0C0C07398F
node --no-warnings wasi-run.mjs target/wasm32-wasip1/release/idgen.wasm -t cuid1   # panics: unsupported
```

To reproduce the browser-target clock trap, build a `cdylib` exporting a
function that calls `SystemTime::now()`, target `wasm32-unknown-unknown`, and
instantiate it with plain `WebAssembly.instantiate` — no imports needed. It traps
with `RuntimeError: unreachable`.

## Scope for 1.6.0

1. Add `wasm32-wasip1` to the release workflow matrix so a `.wasm` ships as a
   release asset. `publish.sh` derives its asset list from the workflow, so the
   manifests follow automatically.
2. Handle cuid1 on wasm — reject with a clear message rather than panicking.
3. Document the WASI artefact in the README, including how to run it.
4. Measure `wasm-opt` and apply it if the win is real.

Version: **1.6.0**, not 1.5.1. A new build target is a feature, not a bug fix —
see [Version numbers](RELEASING.md#version-numbers).

## Open questions

- Is 2.32 MB acceptable for the playground, before or after `wasm-opt`?
- cuid1: reject, or random fingerprint? (Recommendation: reject.)
- Does the playground use the WASI shim, or wait for a wasm-bindgen build?
- Should the `.wasm` also be published somewhere installable (npm?), or is a
  GitHub release asset enough?
