Goal: implement the Stateless History Node v0.1 **gradually**, following:
- `PRD.md` (contract + test gates)
- `ROADMAP.md` (checkbox progress)
- `SPEC.md` (scope + non-goals)

Process rules (strict):
- Work on **one small, verifiable slice** at a time (ideally 1 checkbox or a tight subset).
- After each slice:
  - run the **narrowest verification** possible (e.g. `cargo test`/`cargo build`; plus a smoke RPC call if RPC changed)
  - update progress by checking off the relevant items in `ROADMAP.md` (and add a 1-line “Verified: …” note if helpful)
  - summarize: what changed, what commands you ran + outcome, what’s next
  - include a human-readable **“What changed / What works / What doesn’t yet”** section
  - propose a git commit message, but **DO NOT commit** (the user will commit)

Constraints:
- Minimum code, no scope creep beyond v0.1.
- Do not modify `harness/` - it's a frozen project already. We will work in a new folder creating everything from scratch.
- Prefer `tokio` + `tracing` patterns similar to `harness/`.
- **Dependencies / repo hygiene**:
  - Treat `reth/` and `rindexer/` folders in the repo as **temporary references** (read/search them, but do not depend on them via Cargo path deps).
  - If you need reth crates, consume them as **git dependencies pinned to a commit** (no `path = "../reth/..."`).
  - Pin to the commit hash currently checked out in the local `reth/` folder (use `git -C reth rev-parse HEAD`) so our build matches the verified KB.
  - The new `node/` crate must build without requiring a local `reth/` checkout.
- **v0.1.0 storage**: commit to MDBX now using reth’s storage/db crates; keep it minimal (meta table only: schema version + chain id + config).
- **v0.1.0 RPC**: use `jsonrpsee` directly with a minimal `RpcModule`; do NOT enable reth’s full `eth_` module stack.
- **RPC error semantics**:
  - If a method is entirely unimplemented/unregistered, return standard JSON-RPC **`-32601 Method not found`**.
  - If a method is implemented but a specific params mode is unsupported (e.g. `eth_getBlockByNumber(_, true)`), return JSON-RPC **`-32602 Invalid params`** with a clear message.
- If something fails due to sandbox/network/deps (e.g. fetching git deps), stop and report the exact failing command + error so the user can rerun unsandboxed.

Start with `v0.1.0 Product contract + architecture skeleton`:
- Create a new Rust crate at `node/` (binary).
- Create module skeletons: `cli/`, `rpc/`, `storage/`, `chain/`, `sync/`, `p2p/` (stubs are fine).
- Implement minimal config: chain id (default 1), data dir, RPC bind (default `127.0.0.1:8545`).
- Implement minimal storage bootstrap (MDBX): create/open the DB location and persist schema version + chain id (no ingestion yet).
- Implement minimal JSON-RPC server exposing `eth_chainId`. All other RPC methods return a clear “not supported” error.
- Verify:
  - `cargo test` (or `cargo build`) for `node/`
  - run the node and `curl` a JSON-RPC request to `eth_chainId` and confirm the response
- Tick the corresponding `v0.1.0` checkboxes in `ROADMAP.md`, suggest a commit message, then stop.

Tools available to you:
- `reth` is fully checked out in the root folder of the project for you to reference and search. Each subfolder in `reth/crates` has AGENTS.md summaries.
- `rindexer` is also checked out in the root, with the same AGENTS.md structure. Use it for RPC contract expectations and behavior checks.
- `harness` folder has already working harness that syncs with ethereum nodes and fetches blocks and receipts. It was an initial test that this is at all possible, but maybe you can check how some things are done there, if needed.

Please use best practices everywhere.

If you would have any questions on the direction to take, or any problems you're struggling with - please stop and ask me for advice - we will brainstorm the problem together and choose the best path.