# Maintenance review and validation

This review covered the shared trie, Rust games and CLIs, local arcade server,
and React interface. It does not establish that every possible defect has been
eliminated. Earlier lab reports contain historical benchmarks and architecture
notes; the current code and notes below take precedence.

## Correctness and persistence

- JSON parameters now use exact object keys and types. Escapes, Unicode,
  malformed input, duplicate keys, nesting depth, and finite numbers are checked
  by the existing parser, shared with Wordle statistics. User-supplied error text
  is escaped when serialized.
- Arcade statistics reload from their respective game objects. Boggle stores
  its accumulated score and best words. Older files containing only its rounded
  average recover the closest total; lost precision cannot be reconstructed.
- Synced sibling-file writes replace statistics in a single rename. The previous
  Windows backup sequence could temporarily remove the destination. Failed
  writes preserve the old file and clean up temporary output. Arcade statistics
  commit to memory only after a successful save. Invalid existing files are
  preserved and reported rather than treated as empty statistics.
- Settings persist across backend restarts. The interface reports save failures.
- Finished Boggle rounds are recorded once, including when replaced. Nonempty
  unfinished Spelling Bee puzzles are recorded when replaced; completed puzzles
  are recorded immediately. Empty discarded Bee puzzles do not count as played.
- Custom Wordle secrets must belong to the curated answer pool, keeping guesses
  legal and candidate hints consistent with the secret.

## Runtime and interface

- Boggle's 180-second deadline is enforced in Rust. Late guesses cannot score;
  returning to a page or reloading the frontend does not extend a round. Expired
  rounds are finalized on the next API request, without a separate scheduler.
- The server bounds headers and bodies, rejects ambiguous request framing, uses
  socket timeouts and a bounded worker queue, and restricts browser origins to
  local dev/preview addresses. It remains a single-player loopback server.
- Games start when first opened. Only the visible game consumes typing;
  editable controls, dialogs, and native button activation retain their keys.
- Boggle's release listener reads the latest game state. Pointer capture supports
  dragging outside the board; taps extend the path without the synthetic click
  removing the tile. Duplicate in-flight submissions are blocked.
- Statistics refresh when opened. Initial connection failures expose retry
  controls. Dialogs have accessible names, focus management, Escape support,
  and Tab trapping. Reduced motion affects CSS and celebrations.
- Vite proxies same-origin API requests in dev and preview. The interface retains
  useful error messages from non-successful HTTP responses.
- The Windows launcher builds before starting, installs missing frontend
  dependencies, waits for both endpoints, reports startup failures, and stops its
  own server process trees when Enter is pressed. It does not change execution
  policy; use manual startup if local policy prevents script use.

## Trie and CLI

Trie lookups no longer allocate a normalized copy. Removal, pruning, and cleanup
use iterative traversal so long valid words do not exhaust the call stack.
`completions(prefix, limit)` adds bounded alphabetical prefix queries for hints
and future games. Tests cover removal and cleanup of a 20,000-letter path.
Boggle's CLI exits on EOF at setup/replay prompts and rejects input received
after its timer expires. Its blocking terminal read still requires input before
the CLI can display a timeout; the desktop timer updates independently.

## Verification

Run from the workspace root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd desktop
npm test
npm run build
```

Regression tests cover statistics round-trips and corruption preservation,
settings reload, invalid custom answers, duplicate scoring, round expiry,
replacement of active games, failed writes, JSON edge cases, long trie paths,
prefix completions, API serialization, and HTTP error messages.

In the managed Windows environment used for this review, filesystem tests need
their temporary directory inside the writable workspace:

```powershell
New-Item -ItemType Directory -Force target/test-tmp | Out-Null
$env:TEMP = (Resolve-Path target/test-tmp).Path
$env:TMP = $env:TEMP
cargo test --workspace
```

Browser verification stopped because the computer-use tool could not reliably
establish Chrome's current URL for its policy checks. Live keyboard, pointer,
layout, and modal behavior still need a manual browser check. The Windows
launcher was syntax-checked; its complete interactive lifecycle was not
exercised. Test servers were stopped after the attempt.

The broad dictionary has no proper-noun metadata or difficulty labels. Game
sessions remain in memory, and a server restart discards unfinished play. Game
operations share one state lock; expensive hints can delay other requests.
Multi-user hosting would require session isolation and a different design.
