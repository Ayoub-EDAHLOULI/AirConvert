use std::sync::atomic::{AtomicBool, Ordering};

/// A single global cancellation flag shared by every converter. Only one
/// batch conversion can run at a time in this app (the UI disables the
/// Convert button while one is in flight), so one flag is enough — no need
/// to track per-batch identity.
///
/// Cancellation is checked between files, not mid-file: a pure-Rust
/// conversion (Images, Spreadsheets) has no natural interruption point
/// inside a single file's work, and stopping a sidecar (FFmpeg, Pandoc)
/// mid-encode would leave a corrupt partial output file behind. Stopping
/// cleanly after the current file finishes is simpler and consistent
/// across every converter.
pub struct CancellationState {
    cancelled: AtomicBool,
}

impl Default for CancellationState {
    fn default() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
        }
    }
}

impl CancellationState {
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[tauri::command]
pub fn cancel_conversion(state: tauri::State<CancellationState>) {
    state.cancel();
}
