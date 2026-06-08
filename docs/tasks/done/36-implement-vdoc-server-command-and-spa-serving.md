---
id: 36
title: Implement vdoc server command and SPA serving
kind: task
type: feature
status: done
priority: medium
specs:
- 10
- 11
- 13
designs:
- 34
depends_on:
- 27
- 28
- 29
started_at: 2026-06-08
completed_at: 2026-06-08
---

## Goal

Provide a runnable `vdoc server` command so users can open the local Web UI and
exercise the server APIs from a browser or HTTP client.

## Scope

- Add a `vdoc server` CLI command.
- Support `--host`, `--port`, and `--json` options.
- Bind to `127.0.0.1` by default.
- Start the Axum API router from `vibe-doc-server`.
- Embed `apps/web/dist` SPA assets in the Rust binary when the directory exists
  at compile time.
- Serve embedded SPA assets for `/` and non-API browser routes.
- Fall back to a clear local-development response when embedded SPA assets are
  not available.
- Return startup information in human-readable and JSON output.

## Out of Scope

- Implementing new API endpoints beyond the existing server router.
- Running `pnpm build` from Cargo build scripts or `vdoc server` startup.
- Requiring Node or pnpm for normal Rust-only builds.
- Serving arbitrary filesystem paths as SPA assets.
- Web UI feature work.
- Codex or agent run APIs.
- Arbitrary file serving outside the approved SPA asset directory.

## Checklist

- [x] CLI args parse `vdoc server --host --port --json`.
- [x] Server binds to `127.0.0.1` by default.
- [x] API routes are reachable through the running server.
- [x] `apps/web/dist` assets are embedded when present at compile time.
- [x] Embedded `index.html`, JavaScript, CSS, and static assets are served with
      appropriate content types.
- [x] SPA browser routes fall back to embedded `index.html`.
- [x] Missing embedded SPA assets produce a useful local-development response.
- [x] Startup output includes the listening URL.

## Done Criteria

- [x] CLI/server tests cover successful startup wiring where practical.
- [x] Invalid host or port inputs fail with stable diagnostics.
- [x] Embedded asset lookup rejects path traversal and API route fallthrough.
- [x] Rust-only builds succeed when `apps/web/dist` is absent.
- [x] Related specs remain accurate.

## Result

Implemented vdoc server with host/port/json startup output, Axum API serving, compile-time optional embedded SPA assets, SPA browser-route fallback, API fallthrough protection, and server/CLI regression coverage.
