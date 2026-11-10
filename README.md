# Karabasan

A tribute to a rude, scripted Turkish DOS chat program: questions, wordplay,
occupation-specific teasing, jokes, two number games, and an abrupt goodbye.
The original CP857 dialogue is decoded to UTF-8. Spelling and slang are preserved.

**One Rust conversation engine runs in the terminal and in the browser as WASM.**
The browser's JavaScript is only a renderer and memory/keyboard adapter. There is
no backend, AI service, network conversation, persistence, or runtime dependency
on the DOS executable. The Rust package has no third-party dependencies.

## Terminal

Requires a Rust toolchain with Cargo.

```sh
cargo run --release
```

Options:

```sh
cargo run --release -- --instant          # skip typewriter effects
cargo run --release -- --seed 42          # repeatable random choices
cargo run --release -- --seed 42 --json   # JSON turns; useful for automation
```

Type `/quit` to end a session early. EOF also exits. Answer yes/no questions with
`e` / `h` or `evet` / `hayır`. In the second game, reply `y` for higher, `d` for
lower, or press Enter when Karabasan guesses correctly.

## Browser

```sh
rustup target add wasm32-unknown-unknown
make serve
```

Open **http://127.0.0.1:8080**. `make web` builds the WASM and copies it into
`web/`; deploy that entire directory to any static HTTP server. No bundler,
wasm-bindgen, npm install, or web framework is needed. Serve over HTTP rather
than opening `index.html` directly from disk.

Use `?seed=42` for a repeatable session. The page is a bare black terminal with
green text and a locally bundled, raster-style VT323 font. Enter submits input;
F2 restarts, including during typing. Reduced-motion preferences skip animation.
The interface supports mobile keyboards and screen-reader labels.

VT323 is by Peter Hull / The VT323 Project Authors, distributed under the SIL
Open Font License 1.1; see `web/VT323-OFL.txt`. No external font service is used.

## Verification

```sh
make test
make check
```

`make test` needs Node.js (tested with 26) for real WASM/native transcript parity.
It runs six Rust tests, including 2,000 complete reverse-guessing games, then
compares 30 full native/WASM sessions (1,831 turns) and stresses the ABI with 100
restarts and long Unicode input. `make check` runs formatting and native/WASM
Clippy checks. Chromium was additionally exercised through REA MCP for WASM
loading, input, quitting, and restarting.

A repeatable full mobile-browser check is available when Playwright is installed
and `make serve` is running:

```sh
PLAYWRIGHT_MODULE=/absolute/path/to/playwright-core/index.mjs node tools/browser.mjs
```

It completes both games, checks text-only rendering of HTML-like input, and
restarts during animation. Tested in Chromium 153 at a 390×844 mobile viewport.

## Project map

| Path | Purpose |
| --- | --- |
| `src/lib.rs` | Shared state machine, Turkish casing, games, seeded RNG, turn protocol |
| `src/dialogue.rs` | Generated original literals, keyed by BASIC data-segment offset |
| `src/main.rs` | Terminal presentation and CLI |
| `src/wasm.rs` | Small WASM ABI: allocation, submission, response access |
| `web/` | Static HTML/CSS and rendering-only JavaScript |
| `tools/extract.py` | Reproducible, execution-free EXEPACK unpacker and literal extractor |
| `tools/parity.mjs` | Native/WASM behavioral parity runner |
| `reverse/` | Reverse-engineering notes, original literal inventory, annotated disassembly |

## Fidelity

This is a source-independent reconstruction, not a byte-for-byte emulator.
Recovered wording, numeric branch cutoffs, broad conversation order, optional
questions, job branches, and both games inform the implementation. Modern input
validation, Turkish Unicode casing, bounded input, contradiction handling, and
the seedable RNG are intentional changes. Exact DOS timing, QuickBASIC RNG
sequences, and every quirky edge case are not claimed equivalent.

One original joke involving sexual abuse of a child is omitted from the performed
dialogue; its bytes remain in the archival extraction. Other original profanity
and abrasive humor are retained.

See [`reverse/README.md`](reverse/README.md) for derivations, evidence, and limits.
