---
id: 36
title: Implement vdoc server command and SPA serving
kind: task
type: feature
status: planned
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

- [ ] CLI args parse `vdoc server --host --port --json`.
- [ ] Server binds to `127.0.0.1` by default.
- [ ] API routes are reachable through the running server.
- [ ] `apps/web/dist` assets are embedded when present at compile time.
- [ ] Embedded `index.html`, JavaScript, CSS, and static assets are served with
      appropriate content types.
- [ ] SPA browser routes fall back to embedded `index.html`.
- [ ] Missing embedded SPA assets produce a useful local-development response.
- [ ] Startup output includes the listening URL.

## Done Criteria

- [ ] CLI/server tests cover successful startup wiring where practical.
- [ ] Invalid host or port inputs fail with stable diagnostics.
- [ ] Embedded asset lookup rejects path traversal and API route fallthrough.
- [ ] Rust-only builds succeed when `apps/web/dist` is absent.
- [ ] Related specs remain accurate.

## Result

Not implemented.
