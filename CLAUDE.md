# libretakt

Collaborative sampler. Rust workspace with a **dual-target architecture**: a native audio/WebSocket server and a WASM web UI.

## Build commands

```bash
make build  # build all targets
make check  # checks all targets with clippy
```

## Dual-target pattern

Every module is gated — check `src/lib.rs` before adding new modules:

```rust
#[cfg(not(target_arch = "wasm32"))]  // native server only
#[cfg(target_arch = "wasm32")]       // WASM client only
```

## Core Design

State is mutated by incoming server messages, **never directly by the UI**. The UI
enqueues outbound commands, which are flushed to the WebSocket at the end of each frame.

## Frame Loop

Every egui frame runs three phases in order:

1. **Inbound** — drain WebSocket events, deserialise, mutate `AppState`
2. **Render** — draw UI from current `AppState`; user actions push to `outbox`
3. **Outbound** — serialise and send all queued commands over the WebSocket

## State

| Struct | Purpose |
|---|---|
| `AppState` | Canonical application data; the single source of truth |
| `UiState` | Transient UI-only state (selections, search input, etc.) |
| `outbox` | Queue of `ClientCommand` values pending transmission |

## Architecture: WASM client

- **UI task**: `wasm_bindgen_futures::spawn_local` running `eframe::WebRunner` on canvas `"the_canvas_id"`
- **WebSocket task**: second `spawn_local` with a reconnect loop; uses `gloo_timers::future::sleep` to yield to the browser event loop (WASM is single-threaded, cooperative)
- **Channel bridge**: `src/app_state.rs` defines `ApplicationState` (held by UI) and `WsChannels` (held by WebSocket task), connected via `futures_channel::mpsc::unbounded` pairs
  - `ws_channels.to_ui.unbounded_send(msg)` → send from WebSocket to UI
  - `self.app_state.from_ws.try_next()` in `update()` → drain in UI each frame

## Architecture: native server

- `axum` WebSocket server on `0.0.0.0:3000`, endpoint at `/ws`
- `handle_socket` uses `tokio::select!` to either receive a message or send a keepalive ping if no messages arrive

## Logging

All logging uses **`tracing`** (not the `log` crate).
Use `tracing::info!`, `tracing::warn!`, etc. at call sites.

## Conventions

- Clippy `-D warnings` is enforced in CI — keep the codebase warning-free
- Each logical fix/change gets its own commit
- `tracing::` macros for logging
- WASM async tasks must `.await` periodically to yield — never spin in a tight synchronous loop
