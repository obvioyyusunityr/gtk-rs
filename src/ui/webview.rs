//src/ui/webview.rs
use gtk::prelude::*;
use gtk::Stack;
use gtk::{gio, glib};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use webkit6::prelude::*;
use webkit6::WebView;

fn to_uri(s: &str) -> String {
    if s.contains("://") {
        return s.to_string();
    }
    let path = match s.strip_prefix("~/") {
        Some(rest) => glib::home_dir().join(rest),
        None => std::path::PathBuf::from(s),
    };
    gio::File::for_path(path).uri().to_string()
}

/// Create a WebView that fills its parent, with the inspector enabled.
pub fn build(url: &str) -> WebView {
    let webview = WebView::new();
    enable_inspector(&webview);
    webview.set_hexpand(true);
    webview.set_vexpand(true);

    let url = url.to_string();
    let loaded = Cell::new(false);
    webview.connect_map(move |webview| {
        if !loaded.replace(true) {
            webview.load_uri(&url);
        }
    });

    webview
}

static DARK: AtomicBool = AtomicBool::new(false);
// Page
const DARK_JS: &str = r#"
(function (on) {
  const old = document.getElementById('__dark_override');
  if (old) old.remove();
  document.querySelectorAll('[data-dark-skip]').forEach(e => e.removeAttribute('data-dark-skip'));
  if (!on || !document.head) return;

  function lum(color) {
    const m = color.match(/[\d.]+/g);
    if (!m) return null;
    const [r, g, b, a = 1] = m.map(Number);
    if (a === 0) return null;
    return (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  }

  function pageIsDark() {
    for (const el of [document.body, document.documentElement]) {
      if (!el) continue;
      const l = lum(getComputedStyle(el).backgroundColor);
      if (l !== null) return l < 0.5;
    }
    const t = document.body && lum(getComputedStyle(document.body).color);
    return t !== null && t !== undefined && t > 0.7;
  }

  if (pageIsDark()) return;

  // Mark small images so they follow the page instead of being re-inverted.
  document.querySelectorAll('img').forEach(img => {
    const w = img.naturalWidth || img.width;
    const h = img.naturalHeight || img.height;
    if (w <= 120 || h <= 120) img.setAttribute('data-dark-skip', '');
  });

  const s = document.createElement('style');
  s.id = '__dark_override';
  s.textContent = `
    html { filter: invert(1) hue-rotate(180deg); background: #fff; }
    img:not([data-dark-skip]), video, canvas, iframe, embed, object,
    [style*="background-image"], [style*="background: url"] {
      filter: invert(1) hue-rotate(180deg);
    }
  `;
  document.head.appendChild(s);
})(ON_PLACEHOLDER);
"#;
pub fn apply_dark(webview: &WebView) {
    let js = DARK_JS.replace("ON_PLACEHOLDER", &DARK.load(Ordering::Relaxed).to_string());
    webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
}

pub fn set_dark(webview: &WebView, enabled: bool) {
    DARK.store(enabled, Ordering::Relaxed);
    apply_dark(webview);
}

fn enable_inspector(webview: &WebView) {
    if let Some(settings) = WebViewExt::settings(webview) {
        settings.set_enable_developer_extras(true);
    }
}

/// The WebView of the page currently shown in `stack`.
pub fn current_webview(stack: &Stack) -> Option<WebView> {
    stack.visible_child()?.downcast::<WebView>().ok()
}
