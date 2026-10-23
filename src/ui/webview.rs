//src/ui/webview.rs
use gtk::gio;
use gtk::prelude::*;
use gtk::Stack;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use webkit6::prelude::*;
use webkit6::WebView;
use webkit6::{UserContentInjectedFrames, UserStyleLevel, UserStyleSheet};

/// Create a WebView that fills its parent, with the inspector enabled.
pub fn build(url: &'static str) -> WebView {
    let webview = WebView::new();
    enable_inspector(&webview);
    webview.set_hexpand(true);
    webview.set_vexpand(true);

    let loaded = Cell::new(false);
    webview.connect_map(move |webview| {
        if !loaded.replace(true) {
            webview.load_uri(url);
        }
    });

    webview.connect_load_changed(|wv, event| {
        if event == webkit6::LoadEvent::Finished {
            apply_dark(wv);
        }
    });

    webview
}

const DARK_CSS: &str = "
html {filter: invert(1) hue-rotate(180deg); background: #fff;}
img, video,canvas,picture {filter: invert(1) hue-rotate(180deg);}
";

static DARK: AtomicBool = AtomicBool::new(false);
// Page
const DARK_JS: &str = r#"
(function (on) {
  const old = document.getElementById('__dark_override');
  if (old) old.remove();                 // remove first so we measure the page's real colors
  if (!on || !document.head) return;

  // Perceived brightness (0..1) of a CSS color string, or null if transparent.
  function lum(color) {
    const m = color.match(/[\d.]+/g);
    if (!m) return null;
    const [r, g, b, a = 1] = m.map(Number);
    if (a === 0) return null;
    return (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  }

  function pageIsDark() {
    // 1. Background of body, then html.
    for (const el of [document.body, document.documentElement]) {
      if (!el) continue;
      const l = lum(getComputedStyle(el).backgroundColor);
      if (l !== null) return l < 0.5;
    }
    // 2. Both transparent: the browser paints white, unless the text is light,
    //    which means the page expects a dark canvas.
    const t = document.body && lum(getComputedStyle(document.body).color);
    return t !== null && t !== undefined && t > 0.7;
  }

  if (pageIsDark()) return;              // already dark, leave it alone

  const s = document.createElement('style');
  s.id = '__dark_override';
  s.textContent = `
    html { filter: invert(1) hue-rotate(180deg); background: #fff; }
    img, video, canvas, picture { filter: invert(1) hue-rotate(180deg); }
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
