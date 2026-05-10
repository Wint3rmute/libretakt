use std::collections::VecDeque;

/// How long each notification is visible (seconds).
const DISPLAY_SECS: f64 = 2.0;

/// How long the fade-out lasts at the end of the display window (seconds).
const FADE_SECS: f64 = 0.4;

/// A FIFO queue of transient notification messages shown in the status bar.
///
/// Each message is displayed for [`DISPLAY_SECS`] seconds, fading out during
/// the final [`FADE_SECS`] seconds, then replaced by the next queued message.
#[derive(Default)]
pub struct NotificationQueue {
    queue: VecDeque<String>,
    /// Timestamp (egui's `ctx.input(|i| i.time)`) when the front message first
    /// became visible. `None` when the queue is empty.
    shown_at: Option<f64>,
}

impl NotificationQueue {
    /// Enqueue a notification. Messages are shown in the order they are pushed.
    pub fn push(&mut self, message: impl Into<String>) {
        self.queue.push_back(message.into());
    }

    /// Advance the queue and return the current message with its opacity.
    ///
    /// Returns `Some((message, alpha))` while a notification is visible, where
    /// `alpha` runs from `1.0` down to `0.0` during the fade window.
    /// Returns `None` when the queue is empty.
    ///
    /// Call once per frame, passing `ctx.input(|i| i.time)`.
    pub fn current(&mut self, now: f64) -> Option<(&str, f32)> {
        loop {
            self.queue.front()?;

            let shown_at = *self.shown_at.get_or_insert(now);
            let elapsed = now - shown_at;

            if elapsed >= DISPLAY_SECS {
                self.queue.pop_front();
                self.shown_at = None;
                continue; // check next entry
            }

            let fade_start = DISPLAY_SECS - FADE_SECS;
            let alpha = if elapsed >= fade_start {
                1.0 - ((elapsed - fade_start) / FADE_SECS) as f32
            } else {
                1.0
            };

            return Some((self.queue.front().unwrap(), alpha));
        }
    }
}
