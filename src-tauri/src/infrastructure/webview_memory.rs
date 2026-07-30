#[cfg(any(windows, test))]
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
#[cfg(windows)]
use std::{thread, time::Duration};

#[cfg(windows)]
use tauri::WindowEvent;
use tauri::{Runtime, WebviewWindow};

#[cfg(windows)]
const INACTIVE_DELAY: Duration = Duration::from_secs(15);

#[cfg(any(windows, test))]
#[derive(Clone, Default)]
struct ActivationEpoch(Arc<AtomicU64>);

#[cfg(any(windows, test))]
impl ActivationEpoch {
    fn advance(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel).wrapping_add(1)
    }

    fn is_current(&self, candidate: u64) -> bool {
        self.0.load(Ordering::Acquire) == candidate
    }
}

#[cfg(windows)]
#[derive(Clone, Copy)]
enum MemoryProfile {
    Normal,
    Low,
}

/// Reduces WebView2's memory target after the window has remained inactive.
///
/// A delayed transition avoids reclaiming and immediately restoring memory for
/// brief focus changes such as opening a native file dialog. WebView2 scripts
/// continue running in both profiles; this is not renderer suspension.
#[cfg(windows)]
pub fn install<R: Runtime>(window: WebviewWindow<R>) {
    let epoch = ActivationEpoch::default();
    let event_window = window.clone();

    window.on_window_event(move |event| {
        let WindowEvent::Focused(focused) = event else {
            return;
        };

        if *focused {
            epoch.advance();
            set_memory_profile(&event_window, MemoryProfile::Normal, None);
            return;
        }

        let pending_epoch = epoch.advance();
        let delayed_epoch = epoch.clone();
        let delayed_window = event_window.clone();
        thread::spawn(move || {
            thread::sleep(INACTIVE_DELAY);
            if delayed_epoch.is_current(pending_epoch) {
                set_memory_profile(
                    &delayed_window,
                    MemoryProfile::Low,
                    Some((delayed_epoch, pending_epoch)),
                );
            }
        });
    });
}

#[cfg(not(windows))]
pub fn install<R: Runtime>(_window: WebviewWindow<R>) {}

#[cfg(windows)]
fn set_memory_profile<R: Runtime>(
    window: &WebviewWindow<R>,
    profile: MemoryProfile,
    guard: Option<(ActivationEpoch, u64)>,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
    };
    use windows_core::Interface;

    let level = match profile {
        MemoryProfile::Normal => COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
        MemoryProfile::Low => COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    };

    if let Err(error) = window.with_webview(move |platform_webview| unsafe {
        if guard
            .as_ref()
            .is_some_and(|(epoch, candidate)| !epoch.is_current(*candidate))
        {
            return;
        }

        let result = platform_webview
            .controller()
            .CoreWebView2()
            .and_then(|webview| webview.cast::<ICoreWebView2_19>())
            .and_then(|webview| webview.SetMemoryUsageTargetLevel(level));

        if let Err(error) = result {
            eprintln!("failed to update WebView2 memory profile: {error}");
        }
    }) {
        eprintln!("failed to access WebView2 for memory profile update: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::ActivationEpoch;

    #[test]
    fn refocus_invalidates_a_pending_low_memory_transition() {
        let epoch = ActivationEpoch::default();
        let pending = epoch.advance();

        epoch.advance();

        assert!(!epoch.is_current(pending));
    }

    #[test]
    fn only_the_latest_inactive_transition_remains_current() {
        let epoch = ActivationEpoch::default();
        let first = epoch.advance();
        let latest = epoch.advance();

        assert!(!epoch.is_current(first));
        assert!(epoch.is_current(latest));
    }
}
