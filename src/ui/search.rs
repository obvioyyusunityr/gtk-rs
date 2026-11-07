use gtk::prelude::*;
use gtk::{
    gio, glib, Align, Box as GtkBox, EventControllerKey, Image, Label, ListBox, Orientation,
    Popover, PositionType, Revealer, ScrolledWindow, SearchEntry, SelectionMode, Stack,
};
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
use webkit6::prelude::*;
use webkit6::{FindOptions, WebView};

use super::pages::PAGES;
use super::sidebar;
use super::webview::current_webview;

/// Your backend. Expected reply: [{ "title": "...", "kind": "Invoice", "url": "http://..." }]
const API: &str = "http://127.0.0.1/api/search";

// ---------- mode parsing (public so tests/ can use it) ----------

#[derive(Debug, PartialEq)]
pub enum Mode<'a> {
    Find(&'a str),
    Jump(&'a str),
    Data(&'a str),
}

pub fn parse(text: &str) -> Mode<'_> {
    if let Some(rest) = text.strip_prefix('>') {
        Mode::Jump(rest.trim())
    } else if let Some(rest) = text.strip_prefix('@') {
        Mode::Data(rest.trim())
    } else {
        Mode::Find(text)
    }
}

// ---------- results dropdown ----------

#[derive(Clone)]
enum Target {
    Page(String),
    Url(String),
}

struct Item {
    title: String,
    subtitle: Option<String>,
    icon: Option<&'static str>,
    target: Target,
}

#[derive(Clone)]
struct Results {
    popover: Popover,
    list: ListBox,
    targets: Rc<RefCell<Vec<Target>>>, // index = row index
}

impl Results {
    fn new(anchor: &SearchEntry) -> Self {
        let list = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .build();
        let scroller = ScrolledWindow::builder()
            .child(&list)
            .min_content_width(300)
            .max_content_height(320)
            .propagate_natural_height(true)
            .build();

        let popover = Popover::new();
        popover.set_child(Some(&scroller));
        popover.set_parent(anchor);
        popover.set_autohide(false); // keep typing in the entry
        popover.set_has_arrow(false);
        popover.set_position(PositionType::Bottom);
        popover.set_can_focus(false);

        Self {
            popover,
            list,
            targets: Rc::default(),
        }
    }

    fn clear(&self) {
        while let Some(c) = self.list.first_child() {
            self.list.remove(&c);
        }
        self.targets.borrow_mut().clear();
    }

    fn hide(&self) {
        self.popover.popdown();
        self.clear();
    }

    fn message(&self, text: &str) {
        self.clear();
        let label = Label::builder()
            .label(text)
            .css_classes(["dim-label"])
            .margin_top(12)
            .margin_bottom(12)
            .build();
        self.list.append(&label);
        self.popover.popup();
    }

    fn set_items(&self, items: Vec<Item>) {
        if items.is_empty() {
            self.message("No results");
            return;
        }
        self.clear();
        for item in items {
            let text = GtkBox::new(Orientation::Vertical, 2);
            text.append(
                &Label::builder()
                    .label(&item.title)
                    .halign(Align::Start)
                    .build(),
            );
            if let Some(sub) = &item.subtitle {
                text.append(
                    &Label::builder()
                        .label(sub)
                        .halign(Align::Start)
                        .css_classes(["dim-label", "caption"])
                        .build(),
                );
            }
            let row = GtkBox::new(Orientation::Horizontal, 10);
            row.set_margin_top(6);
            row.set_margin_bottom(6);
            row.set_margin_start(6);
            if let Some(icon) = item.icon {
                row.append(&Image::from_icon_name(icon));
            }
            row.append(&text);
            self.list.append(&row);
            self.targets.borrow_mut().push(item.target);
        }
        self.list.select_row(self.list.row_at_index(0).as_ref());
        self.popover.popup();
    }

    fn step(&self, delta: i32) {
        let n = self.targets.borrow().len() as i32;
        if n == 0 {
            return;
        }
        let cur = self.list.selected_row().map(|r| r.index()).unwrap_or(-1);
        let next = (cur + delta).clamp(0, n - 1);
        self.list.select_row(self.list.row_at_index(next).as_ref());
    }

    fn selected_target(&self) -> Option<Target> {
        let idx = self.list.selected_row().map(|r| r.index()).unwrap_or(0);
        self.targets.borrow().get(idx as usize).cloned()
    }
}

// ---------- 1. find in page ----------

fn webviews(stack: &Stack) -> Vec<WebView> {
    PAGES
        .iter()
        .filter_map(|p| stack.child_by_name(p.id)?.downcast::<WebView>().ok())
        .collect()
}

fn find(stack: &Stack, text: &str) {
    for wv in webviews(stack) {
        if let Some(fc) = wv.find_controller() {
            fc.search_finish();
        }
    }
    if text.is_empty() {
        return;
    }
    if let Some(fc) = current_webview(stack).and_then(|wv| wv.find_controller()) {
        let opts = (FindOptions::CASE_INSENSITIVE | FindOptions::WRAP_AROUND).bits();
        fc.search(text, opts, 0);
    }
}

// ---------- 3. jump to page ----------

fn page_matches(query: &str) -> Vec<Item> {
    let q = query.to_lowercase();
    PAGES
        .iter()
        .filter(|p| p.title.to_lowercase().contains(&q))
        .map(|p| Item {
            title: p.title.trim().to_string(),
            subtitle: p
                .parent
                .and_then(|id| PAGES.iter().find(|x| x.id == id))
                .map(|x| format!("in {}", x.title.trim())),
            icon: Some(p.icon),
            target: Target::Page(p.id.to_string()),
        })
        .collect()
}

// ---------- 4. app data from the backend ----------

#[derive(Deserialize)]
struct Hit {
    title: String,
    #[serde(default)]
    kind: Option<String>,
    url: String,
}

fn fetch(q: &str) -> Result<Vec<Hit>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .build();
    agent
        .get(API)
        .query("q", q)
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<Vec<Hit>>()
        .map_err(|e| e.to_string())
}

// ---------- wiring ----------

pub fn install(search: &SearchEntry, stack: &Stack, sidebar_root: &Revealer) {
    let results = Results::new(search);
    let generation = Rc::new(Cell::new(0u64)); // drops stale async answers
    let filtering = Rc::new(Cell::new(false));

    // Open a result. Deferred so we don't clear the list from inside its own signal.
    let open: Rc<dyn Fn(Target)> = Rc::new({
        let (search, stack) = (search.clone(), stack.clone());
        move |target| {
            let (search, stack) = (search.clone(), stack.clone());
            glib::idle_add_local_once(move || {
                search.set_text(""); // resets filter + highlights first
                match target {
                    Target::Page(id) => stack.set_visible_child_name(&id),
                    Target::Url(url) => {
                        let wv = current_webview(&stack).or_else(|| {
                            stack.set_visible_child_name("home");
                            current_webview(&stack)
                        });
                        if let Some(wv) = wv {
                            wv.load_uri(&url);
                        }
                    }
                }
            });
        }
    });

    // Typing.
    search.connect_search_changed({
        let (stack, root, results) = (stack.clone(), sidebar_root.clone(), results.clone());
        let (generation, filtering) = (generation.clone(), filtering.clone());
        move |entry| {
            let text = entry.text();
            generation.set(generation.get() + 1);
            let my_gen = generation.get();

            find(&stack, ""); // always drop old highlights

            match parse(&text) {
                Mode::Find(q) => {
                    if filtering.replace(false) {
                        sidebar::filter(&root, ""); // 2. undo sidebar filter
                    }
                    results.hide();
                    find(&stack, q);
                }
                Mode::Jump(q) => {
                    filtering.set(true);
                    sidebar::filter(&root, q); // 2. filter the sidebar as you type
                    results.set_items(page_matches(q));
                }
                Mode::Data(q) => {
                    if filtering.replace(false) {
                        sidebar::filter(&root, "");
                    }
                    if q.chars().count() < 2 {
                        results.message("Type at least 2 characters");
                        return;
                    }
                    results.message("Searching…");
                    let q = q.to_string();
                    let (results, generation) = (results.clone(), generation.clone());
                    glib::timeout_add_local_once(Duration::from_millis(250), move || {
                        if generation.get() != my_gen {
                            return; // typed again during the debounce
                        }
                        glib::spawn_future_local(async move {
                            let res = gio::spawn_blocking(move || fetch(&q)).await;
                            if generation.get() != my_gen {
                                return;
                            }
                            match res {
                                Ok(Ok(hits)) => results.set_items(
                                    hits.into_iter()
                                        .map(|h| Item {
                                            title: h.title,
                                            subtitle: h.kind,
                                            icon: None,
                                            target: Target::Url(h.url),
                                        })
                                        .collect(),
                                ),
                                Ok(Err(e)) => results.message(&format!("Search failed: {e}")),
                                Err(_) => results.message("Search failed"),
                            }
                        });
                    });
                }
            }
        }
    });

    // Enter: next match, or open the selected result.
    search.connect_activate({
        let (stack, results, open) = (stack.clone(), results.clone(), open.clone());
        move |entry| match parse(&entry.text()) {
            Mode::Find(_) => {
                if let Some(fc) = current_webview(&stack).and_then(|wv| wv.find_controller()) {
                    fc.search_next();
                }
            }
            _ => {
                if let Some(t) = results.selected_target() {
                    open(t);
                }
            }
        }
    });

    // Ctrl+G / Ctrl+Shift+G for next / previous find match.
    search.connect_next_match({
        let stack = stack.clone();
        move |_| {
            if let Some(fc) = current_webview(&stack).and_then(|wv| wv.find_controller()) {
                fc.search_next();
            }
        }
    });
    search.connect_previous_match({
        let stack = stack.clone();
        move |_| {
            if let Some(fc) = current_webview(&stack).and_then(|wv| wv.find_controller()) {
                fc.search_previous();
            }
        }
    });

    // Escape: clear everything.
    search.connect_stop_search(|entry| entry.set_text(""));

    // Click a result.
    results.list.connect_row_activated({
        let (results, open) = (results.clone(), open.clone());
        move |_, row| {
            if let Some(t) = results.targets.borrow().get(row.index() as usize).cloned() {
                open(t);
            }
        }
    });

    // Up / Down move through results.
    let keys = EventControllerKey::new();
    keys.connect_key_pressed({
        let results = results.clone();
        move |_, key, _, _| {
            if !results.popover.is_visible() {
                return glib::Propagation::Proceed;
            }
            match key {
                gtk::gdk::Key::Down => results.step(1),
                gtk::gdk::Key::Up => results.step(-1),
                _ => return glib::Propagation::Proceed,
            }
            glib::Propagation::Stop
        }
    });
    search.add_controller(keys);

    // Switching pages while in find mode: search the new page.
    stack.connect_visible_child_notify({
        let (stack, search) = (stack.clone(), search.clone());
        move |_| {
            if let Mode::Find(q) = parse(&search.text()) {
                find(&stack, q);
            }
        }
    });
}
