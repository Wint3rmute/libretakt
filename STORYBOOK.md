# Libretakt UI Storybook

Visual reference for every tested UI state. Screenshots are generated automatically by the snapshot tests in [`tests/snapshots.rs`](tests/snapshots.rs) — run `UPDATE_SNAPSHOTS=1 cargo test --test snapshots` to regenerate them.

---

## Connecting (default view)

The initial state shown to every client before the server has responded. The app opens a WebSocket connection and displays a placeholder while waiting for the server's `Init` message.

| Dark | Light |
|------|-------|
| ![Default view — dark](tests/snapshots/app_default_view_dark.png) | ![Default view — light](tests/snapshots/app_default_view_light.png) |

---

## Player selection

Shown immediately after the server sends its `Init` message. The client has been assigned an ID and received the full sequencer state. The player-selection grid lets the user navigate to any of the eight tracks or the mixing console.

| Dark | Light |
|------|-------|
| ![Player selection — dark](tests/snapshots/app_connected_player_selection_dark.png) | ![Player selection — light](tests/snapshots/app_connected_player_selection_light.png) |

---

## Sequencer view (track 1, locked)

The track-1 sequencer view, reached by pressing **Sequencer** on the player-selection screen. This scenario has a four-on-the-floor pattern active (steps 1, 5, 9, 13) and the lock held by the current client, so step buttons are rendered with full-strength text and are interactive. The top section shows the four per-track parameter sliders (Filter, Resonance, Volume, Pan).

| Dark | Light |
|------|-------|
| ![Sequencer view — dark](tests/snapshots/app_sequencer_view_dark.png) | ![Sequencer view — light](tests/snapshots/app_sequencer_view_light.png) |

---

## Track lock states

Player-selection screen with mixed lock ownership across the tracks. Track 1 is locked by **this client**, track 2 is locked by a **peer**, and tracks 3–4 are free. This exercises the lock-indicator rendering for all three ownership states simultaneously.

| Dark | Light |
|------|-------|
| ![Track lock states — dark](tests/snapshots/app_track_lock_states_dark.png) | ![Track lock states — light](tests/snapshots/app_track_lock_states_light.png) |

---

## Disconnected

The state after a previously-established connection is lost. The app transitions back to `Disconnected` and shows a "Reconnecting…" notification in the bottom panel, which fades out over time. A reconnect attempt will be made automatically.

| Dark | Light |
|------|-------|
| ![Disconnected — dark](tests/snapshots/app_disconnected_dark.png) | ![Disconnected — light](tests/snapshots/app_disconnected_light.png) |
