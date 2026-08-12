use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use crossterm::event::{Event as CtEvent, KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use tui_tree_widget::{TreeItem, TreeState};

use crate::action::Action;
use crate::db;
use crate::db::connection::SharedConnection;
use crate::event::AppMsg;
use crate::http;
use crate::keymap;
use crate::mode::{Focus, Mode, SidebarView};
use crate::model::auth::{ApiKeyLocation, AuthConfig};
use crate::model::collection::CollectionNode;
use crate::model::history::{HistoryEntry, NewHistoryEntry};
use crate::model::request::{BodyKind, KeyValue, Method, RequestFile};
use crate::model::response::ResponseData;
use crate::storage;
use crate::ui::widgets::{self, BodyView};

/// How long to wait for a response before giving up. Long enough to
/// tolerate a slow-but-legitimate endpoint (cold starts, heavy queries),
/// short enough that a dead connection doesn't hang the app indefinitely.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub struct App {
    pub mode: Mode,
    pub focus: Focus,
    pub sidebar_view: SidebarView,
    pub url_input: Input,
    pub command_input: Input,
    pub response: Option<ResponseData>,
    pub response_error: Option<String>,
    pub in_flight: bool,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub collection_root: PathBuf,
    pub tree_items: Vec<TreeItem<'static, PathBuf>>,
    pub tree_state: TreeState<PathBuf>,
    pub current_request: Option<RequestFile>,
    pub current_path: Option<PathBuf>,
    pub auth_kind: AuthKind,
    pub auth_field: usize,
    pub auth_bearer_token: Input,
    pub auth_basic_username: Input,
    pub auth_basic_password: Input,
    pub auth_apikey_name: Input,
    pub auth_apikey_value: Input,
    pub auth_apikey_location: ApiKeyLocation,
    pub history_entries: Vec<HistoryEntry>,
    pub history_selected: usize,
    pub response_tree_state: TreeState<usize>,
    pub search_input: Input,
    pub search_matches: Vec<Vec<usize>>,
    pub search_match_index: usize,
    pending_g: bool,
    history_conn: Option<SharedConnection>,
    http_client: reqwest::Client,
    app_tx: UnboundedSender<AppMsg>,
}

impl App {
    pub fn new(app_tx: UnboundedSender<AppMsg>, collection_root: PathBuf) -> Self {
        let (history_conn, history_status) = match db::connection::open(&collection_root) {
            Ok(conn) => (Some(conn), None),
            Err(err) => (None, Some(format!("History unavailable: {err}"))),
        };

        let mut app = Self {
            mode: Mode::Normal,
            focus: Focus::Sidebar,
            sidebar_view: SidebarView::Collections,
            url_input: Input::default(),
            command_input: Input::default(),
            response: None,
            response_error: None,
            in_flight: false,
            should_quit: false,
            status_message: history_status,
            collection_root,
            tree_items: Vec::new(),
            tree_state: TreeState::default(),
            current_request: None,
            current_path: None,
            auth_kind: AuthKind::None,
            auth_field: 0,
            auth_bearer_token: Input::default(),
            auth_basic_username: Input::default(),
            auth_basic_password: Input::default(),
            auth_apikey_name: Input::default(),
            auth_apikey_value: Input::default(),
            auth_apikey_location: ApiKeyLocation::Header,
            history_entries: Vec::new(),
            history_selected: 0,
            response_tree_state: TreeState::default(),
            search_input: Input::default(),
            search_matches: Vec::new(),
            search_match_index: 0,
            pending_g: false,
            history_conn,
            http_client: reqwest::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("failed to build HTTP client"),
            app_tx,
        };
        app.refresh_tree();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Insert => self.handle_insert_key(key),
            Mode::Command => self.handle_command_key(key),
            Mode::Search => self.handle_search_key(key),
            Mode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        // `gg` is a two-keystroke vim chord (jump to top); everything else
        // is a single keypress resolved through the keymap table.
        if key.code == KeyCode::Char('g') {
            if self.pending_g {
                self.pending_g = false;
                self.status_message = None;
                self.dispatch(Action::JumpTop);
            } else {
                self.pending_g = true;
            }
            return;
        }
        self.pending_g = false;

        self.status_message = None;
        if let Some(action) = keymap::resolve(self.mode, self.focus, key) {
            self.dispatch(action);
        }
    }

    fn handle_insert_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter if self.focus == Focus::UrlBar => {
                self.mode = Mode::Normal;
                self.send_request();
            }
            _ => {
                if let Some(input) = self.active_text_input() {
                    input.handle_event(&CtEvent::Key(key));
                }
            }
        }
    }

    /// The text-editing widget the current focus/field points at, if any
    /// (the sidebar and the API-key location field aren't free text).
    fn active_text_input(&mut self) -> Option<&mut Input> {
        match self.focus {
            Focus::UrlBar => Some(&mut self.url_input),
            Focus::Auth => match (self.auth_kind, self.auth_field) {
                (AuthKind::Bearer, 0) => Some(&mut self.auth_bearer_token),
                (AuthKind::Basic, 0) => Some(&mut self.auth_basic_username),
                (AuthKind::Basic, 1) => Some(&mut self.auth_basic_password),
                (AuthKind::ApiKey, 0) => Some(&mut self.auth_apikey_name),
                (AuthKind::ApiKey, 1) => Some(&mut self.auth_apikey_value),
                _ => None,
            },
            Focus::Sidebar | Focus::Response => None,
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_input.reset();
            }
            KeyCode::Enter => {
                let cmd = self.command_input.value().to_string();
                self.command_input.reset();
                self.mode = Mode::Normal;
                self.run_command(&cmd);
            }
            _ => {
                self.command_input.handle_event(&CtEvent::Key(key));
            }
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_input.reset();
            }
            KeyCode::Enter => {
                let query = self.search_input.value().to_string();
                self.mode = Mode::Normal;
                self.perform_search(&query);
            }
            _ => {
                self.search_input.handle_event(&CtEvent::Key(key));
            }
        }
    }

    fn run_command(&mut self, cmd: &str) {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return;
        }
        let mut parts = cmd.splitn(2, ' ');
        let verb = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();
        match verb {
            "q" | "quit" => self.should_quit = true,
            "w" | "write" => self.save_current(),
            "new" => self.new_request(rest),
            "history" => self.show_history(),
            "find" => self.search_history(rest),
            "env" => {
                self.status_message =
                    Some("Environment variables aren't in this release yet — planned for a future version".to_string())
            }
            "collections" => {
                self.sidebar_view = SidebarView::Collections;
                self.focus = Focus::Sidebar;
            }
            other => self.status_message = Some(format!("Unknown command: {other}")),
        }
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::EnterInsert => self.mode = Mode::Insert,
            Action::EnterCommand => self.mode = Mode::Command,
            Action::SendRequest => self.send_request(),
            Action::ToggleFocus => self.toggle_focus(),
            Action::ToggleFocusBack => self.toggle_focus_back(),
            Action::TreeDown => match self.focus {
                Focus::Response => {
                    self.response_tree_state.key_down();
                }
                _ => self.move_selection(1),
            },
            Action::TreeUp => match self.focus {
                Focus::Response => {
                    self.response_tree_state.key_up();
                }
                _ => self.move_selection(-1),
            },
            Action::TreeCollapse => match self.focus {
                Focus::Response => {
                    self.response_tree_state.key_left();
                }
                _ => {
                    if self.sidebar_view == SidebarView::Collections {
                        self.tree_state.key_left();
                    }
                }
            },
            Action::TreeExpand => match self.focus {
                Focus::Response => {
                    self.response_tree_state.key_right();
                }
                _ => {
                    if self.sidebar_view == SidebarView::Collections {
                        self.tree_state.key_right();
                    }
                }
            },
            Action::TreeOpen => self.open_selected(),
            Action::SearchStart => {
                self.mode = Mode::Search;
                self.search_input.reset();
            }
            Action::SearchNext => self.jump_to_match(1),
            Action::SearchPrev => self.jump_to_match(-1),
            Action::JumpTop => match self.focus {
                Focus::Sidebar => match self.sidebar_view {
                    SidebarView::Collections => {
                        self.tree_state.select_first();
                    }
                    SidebarView::History => self.history_selected = 0,
                },
                Focus::Response => {
                    self.response_tree_state.select(vec![0]);
                }
                Focus::UrlBar | Focus::Auth => {}
            },
            Action::JumpBottom => match self.focus {
                Focus::Sidebar => match self.sidebar_view {
                    SidebarView::Collections => {
                        for _ in 0..2000 {
                            self.tree_state.key_down();
                        }
                    }
                    SidebarView::History => {
                        self.history_selected = self.history_entries.len().saturating_sub(1);
                    }
                },
                Focus::Response => {
                    for _ in 0..2000 {
                        self.response_tree_state.key_down();
                    }
                }
                Focus::UrlBar | Focus::Auth => {}
            },
            Action::AuthCycleKind => {
                self.auth_kind = self.auth_kind.next();
                self.auth_field = 0;
            }
            Action::AuthFieldDown => {
                let count = self.auth_kind.field_count();
                if count > 0 {
                    self.auth_field = (self.auth_field + 1).min(count - 1);
                }
            }
            Action::AuthFieldUp => self.auth_field = self.auth_field.saturating_sub(1),
            Action::AuthToggleLocation => {
                if self.auth_kind == AuthKind::ApiKey && self.auth_field == 2 {
                    self.auth_apikey_location = match self.auth_apikey_location {
                        ApiKeyLocation::Header => ApiKeyLocation::Query,
                        ApiKeyLocation::Query => ApiKeyLocation::Header,
                    };
                }
            }
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::UrlBar,
            Focus::UrlBar => Focus::Auth,
            Focus::Auth => Focus::Response,
            Focus::Response => Focus::Sidebar,
        };
    }

    fn toggle_focus_back(&mut self) {
        self.focus = match self.focus {
            Focus::UrlBar => Focus::Sidebar,
            Focus::Auth => Focus::UrlBar,
            Focus::Response => Focus::Auth,
            Focus::Sidebar => Focus::Response,
        };
    }

    fn move_selection(&mut self, delta: i32) {
        match self.sidebar_view {
            SidebarView::Collections => {
                if delta > 0 {
                    self.tree_state.key_down();
                } else {
                    self.tree_state.key_up();
                }
            }
            SidebarView::History => {
                if self.history_entries.is_empty() {
                    return;
                }
                let max = self.history_entries.len() - 1;
                self.history_selected = if delta > 0 {
                    (self.history_selected + 1).min(max)
                } else {
                    self.history_selected.saturating_sub(1)
                };
            }
        }
    }

    fn open_selected(&mut self) {
        match self.sidebar_view {
            SidebarView::Collections => self.tree_open_selected(),
            SidebarView::History => self.rerun_selected_history(),
        }
    }

    fn tree_open_selected(&mut self) {
        let Some(path) = self.tree_state.selected().last().cloned() else {
            return;
        };
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            self.load_request(path);
        } else {
            self.tree_state.toggle_selected();
        }
    }

    fn refresh_tree(&mut self) {
        let nodes = storage::fs_tree::build_tree(&self.collection_root).unwrap_or_default();
        self.tree_items = CollectionNode::to_tree_items(&nodes);
    }

    fn selected_dir(&self) -> PathBuf {
        match self.tree_state.selected().last() {
            Some(path) if path.extension().and_then(|e| e.to_str()) == Some("toml") => path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| self.collection_root.clone()),
            Some(path) => path.clone(),
            None => self.collection_root.clone(),
        }
    }

    fn open_request(&mut self, path: Option<PathBuf>, request: RequestFile) {
        self.url_input = Input::new(request.url.clone());
        self.load_auth_fields(&request.auth);
        self.current_request = Some(request);
        self.current_path = path;
    }

    fn load_auth_fields(&mut self, auth: &AuthConfig) {
        self.auth_bearer_token = Input::default();
        self.auth_basic_username = Input::default();
        self.auth_basic_password = Input::default();
        self.auth_apikey_name = Input::default();
        self.auth_apikey_value = Input::default();
        self.auth_apikey_location = ApiKeyLocation::Header;
        self.auth_field = 0;
        self.auth_kind = match auth {
            AuthConfig::None => AuthKind::None,
            AuthConfig::Bearer { token } => {
                self.auth_bearer_token = Input::new(token.clone());
                AuthKind::Bearer
            }
            AuthConfig::Basic { username, password } => {
                self.auth_basic_username = Input::new(username.clone());
                self.auth_basic_password = Input::new(password.clone());
                AuthKind::Basic
            }
            AuthConfig::ApiKey {
                name,
                value,
                location,
            } => {
                self.auth_apikey_name = Input::new(name.clone());
                self.auth_apikey_value = Input::new(value.clone());
                self.auth_apikey_location = location.clone();
                AuthKind::ApiKey
            }
        };
    }

    fn build_auth_config(&self) -> AuthConfig {
        match self.auth_kind {
            AuthKind::None => AuthConfig::None,
            AuthKind::Bearer => AuthConfig::Bearer {
                token: self.auth_bearer_token.value().to_string(),
            },
            AuthKind::Basic => AuthConfig::Basic {
                username: self.auth_basic_username.value().to_string(),
                password: self.auth_basic_password.value().to_string(),
            },
            AuthKind::ApiKey => AuthConfig::ApiKey {
                name: self.auth_apikey_name.value().to_string(),
                value: self.auth_apikey_value.value().to_string(),
                location: self.auth_apikey_location.clone(),
            },
        }
    }

    fn load_request(&mut self, path: PathBuf) {
        match storage::request_file::load_request(&path) {
            Ok(request) => {
                self.status_message = Some(format!("Loaded {}", request.meta.name));
                self.open_request(Some(path), request);
            }
            Err(err) => {
                self.status_message = Some(format!("Failed to load {}: {err}", path.display()));
            }
        }
    }

    fn save_current(&mut self) {
        let Some(path) = self.current_path.clone() else {
            self.status_message = Some("No request open — use :new <name> first".to_string());
            return;
        };
        let mut request = self.current_request.clone().unwrap_or_else(RequestFile::scratch);
        request.url = self.url_input.value().to_string();
        request.auth = self.build_auth_config();
        match storage::request_file::save_request(&path, &request) {
            Ok(()) => {
                self.current_request = Some(request);
                self.status_message = Some(format!("Saved {}", path.display()));
            }
            Err(err) => self.status_message = Some(format!("Save failed: {err}")),
        }
    }

    fn new_request(&mut self, name: &str) {
        if name.is_empty() {
            self.status_message = Some("Usage: :new <name>".to_string());
            return;
        }
        let dir = self.selected_dir();
        let path = dir.join(format!("{}.toml", slugify(name)));
        if path.exists() {
            self.status_message = Some(format!("{} already exists", path.display()));
            return;
        }
        let request = RequestFile::scratch_named(name);
        match storage::request_file::save_request(&path, &request) {
            Ok(()) => {
                self.refresh_tree();
                self.status_message = Some(format!("Created {}", path.display()));
                self.open_request(Some(path), request);
                self.focus = Focus::UrlBar;
                self.mode = Mode::Insert;
            }
            Err(err) => self.status_message = Some(format!("Failed to create request: {err}")),
        }
    }

    fn show_history(&mut self) {
        self.sidebar_view = SidebarView::History;
        self.focus = Focus::Sidebar;
        self.refresh_history();
    }

    fn refresh_history(&mut self) {
        let Some(conn) = self.history_conn.clone() else {
            self.status_message = Some("History database unavailable".to_string());
            return;
        };
        let tx = self.app_tx.clone();
        tokio::task::spawn_blocking(move || {
            let result = db::history_repo::list_recent(&conn, 200).map_err(|e| e.to_string());
            let _ = tx.send(AppMsg::HistoryLoaded(result));
        });
    }

    fn search_history(&mut self, query: &str) {
        self.sidebar_view = SidebarView::History;
        self.focus = Focus::Sidebar;
        if query.is_empty() {
            self.refresh_history();
            return;
        }
        let Some(conn) = self.history_conn.clone() else {
            self.status_message = Some("History database unavailable".to_string());
            return;
        };
        let tx = self.app_tx.clone();
        let query = query.to_string();
        tokio::task::spawn_blocking(move || {
            let result = db::history_repo::search(&conn, &query, 200).map_err(|e| e.to_string());
            let _ = tx.send(AppMsg::HistoryLoaded(result));
        });
    }

    fn rerun_selected_history(&mut self) {
        let Some(entry) = self.history_entries.get(self.history_selected).cloned() else {
            return;
        };

        let mut request = RequestFile::scratch_named(
            entry.request_name.as_deref().unwrap_or("history rerun"),
        );
        request.url = entry.url;
        request.method = parse_method(&entry.method);
        if let Some(headers_json) = &entry.request_headers
            && let Ok(headers) = serde_json::from_str::<Vec<(String, String)>>(headers_json)
        {
            request.headers = headers
                .into_iter()
                .map(|(name, value)| KeyValue {
                    name,
                    value,
                    enabled: true,
                })
                .collect();
        }

        let had_auth = entry.auth_type.is_some();
        self.open_request(None, request);
        self.focus = Focus::UrlBar;
        self.send_request();
        if had_auth {
            self.status_message =
                Some("Note: auth credentials aren't stored in history — re-add them if needed".to_string());
        }
    }

    fn send_request(&mut self) {
        if self.in_flight {
            return;
        }
        let mut request = self.current_request.clone().unwrap_or_else(RequestFile::scratch);
        request.url = self.url_input.value().trim().to_string();
        request.auth = self.build_auth_config();
        if request.url.is_empty() {
            return;
        }
        self.in_flight = true;
        self.response_error = None;
        http::task::spawn_send(self.http_client.clone(), self.app_tx.clone(), request);
    }

    pub fn handle_app_msg(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::HttpResult { request, result } => {
                self.in_flight = false;
                match &result {
                    Ok(response) => {
                        self.response = Some(response.clone());
                        self.response_error = None;
                    }
                    Err(err) => {
                        self.response = None;
                        self.response_error = Some(err.clone());
                    }
                }
                self.response_tree_state = TreeState::default();
                self.search_matches.clear();
                self.search_match_index = 0;
                self.record_history(&request, &result);
            }
            AppMsg::HistoryLoaded(result) => match result {
                Ok(entries) => {
                    self.history_entries = entries;
                    self.history_selected = 0;
                }
                Err(err) => self.status_message = Some(format!("Failed to load history: {err}")),
            },
        }
    }

    fn record_history(&mut self, request: &RequestFile, result: &Result<ResponseData, String>) {
        let Some(conn) = self.history_conn.clone() else {
            return;
        };

        let collection_path = self.current_path.as_ref().map(|p| p.display().to_string());
        let request_headers = serde_json::to_string(
            &request
                .headers
                .iter()
                .filter(|h| h.enabled)
                .map(|h| (h.name.clone(), h.value.clone()))
                .collect::<Vec<_>>(),
        )
        .ok();
        let request_body = match &request.body {
            BodyKind::Raw { content, .. } => Some(content.clone()),
            _ => None,
        };
        let auth_type = match &request.auth {
            AuthConfig::None => None,
            AuthConfig::Bearer { .. } => Some("bearer".to_string()),
            AuthConfig::Basic { .. } => Some("basic".to_string()),
            AuthConfig::ApiKey { .. } => Some("api_key".to_string()),
        };

        let (status_code, response_headers, response_body, response_size_bytes, error_message, duration_ms) =
            match result {
                Ok(response) => (
                    Some(response.status),
                    serde_json::to_string(&response.headers).ok(),
                    Some(response.body.clone()),
                    Some(response.body.len() as i64),
                    None,
                    Some(response.elapsed.as_millis() as i64),
                ),
                Err(err) => (None, None, None, None, Some(err.clone()), None),
            };

        let entry = NewHistoryEntry {
            request_uuid: Some(request.meta.id),
            collection_path,
            request_name: Some(request.meta.name.clone()),
            method: request.method.to_string(),
            url: request.url.clone(),
            request_headers,
            request_body,
            auth_type,
            status_code,
            response_headers,
            response_body,
            response_size_bytes,
            error_message,
            duration_ms,
            sent_at: Utc::now(),
        };

        tokio::task::spawn_blocking(move || {
            let _ = db::history_repo::insert(&conn, &entry);
        });

        if self.sidebar_view == SidebarView::History {
            self.refresh_history();
        }
    }

    fn perform_search(&mut self, query: &str) {
        self.search_matches.clear();
        self.search_match_index = 0;

        let query = query.trim();
        if query.is_empty() {
            return;
        }
        let Some(response) = &self.response else {
            self.status_message = Some("No response to search".to_string());
            return;
        };

        let content_type = response
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.as_str());
        let view = widgets::detect(content_type, &response.body, response.is_valid_utf8);

        let index: Vec<(Vec<usize>, String)> = match view {
            BodyView::Json(value) => widgets::json_tree::build(&value).index,
            BodyView::Xml(body) => match roxmltree::Document::parse(&body) {
                Ok(doc) => widgets::xml_tree::build(&doc).index,
                Err(_) => Vec::new(),
            },
            _ => {
                self.status_message = Some("Search is only available for JSON/XML responses".to_string());
                Vec::new()
            }
        };

        let query_lower = query.to_lowercase();
        self.search_matches = index
            .into_iter()
            .filter(|(_, text)| text.to_lowercase().contains(&query_lower))
            .map(|(path, _)| path)
            .collect();

        if self.search_matches.is_empty() {
            self.status_message = Some(format!("No matches for \"{query}\""));
            return;
        }

        self.status_message = Some(format!("{} match(es) for \"{query}\" (n/N to cycle)", self.search_matches.len()));
        self.select_match(0);
    }

    fn jump_to_match(&mut self, delta: i32) {
        if self.search_matches.is_empty() {
            return;
        }
        let len = self.search_matches.len() as i32;
        let next = (self.search_match_index as i32 + delta).rem_euclid(len);
        self.select_match(next as usize);
    }

    fn select_match(&mut self, index: usize) {
        self.search_match_index = index;
        let Some(path) = self.search_matches.get(index).cloned() else {
            return;
        };
        for len in 1..path.len() {
            self.response_tree_state.open(path[..len].to_vec());
        }
        self.response_tree_state.select(path);
    }

    pub fn on_tick(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    None,
    Bearer,
    Basic,
    ApiKey,
}

impl AuthKind {
    fn next(self) -> Self {
        match self {
            AuthKind::None => AuthKind::Bearer,
            AuthKind::Bearer => AuthKind::Basic,
            AuthKind::Basic => AuthKind::ApiKey,
            AuthKind::ApiKey => AuthKind::None,
        }
    }

    fn field_count(self) -> usize {
        match self {
            AuthKind::None => 0,
            AuthKind::Bearer => 1,
            AuthKind::Basic => 2,
            AuthKind::ApiKey => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AuthKind::None => "None",
            AuthKind::Bearer => "Bearer",
            AuthKind::Basic => "Basic",
            AuthKind::ApiKey => "API Key",
        }
    }
}

fn parse_method(raw: &str) -> Method {
    match raw.to_ascii_uppercase().as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "PATCH" => Method::Patch,
        "DELETE" => Method::Delete,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        other => Method::Custom(other.to_string()),
    }
}

fn slugify(name: &str) -> String {
    let mut slug: String = name
        .trim()
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "request".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_app() -> (App, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let app = App::new(tx, dir.path().to_path_buf());
        (app, dir)
    }

    fn json_response(body: &str) -> ResponseData {
        ResponseData {
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: body.to_string(),
            is_valid_utf8: true,
            byte_len: body.len(),
            elapsed: Duration::from_millis(1),
        }
    }

    #[test]
    fn search_finds_a_match_and_selects_it_in_the_tree() {
        let (mut app, _dir) = test_app();
        app.response = Some(json_response(r#"{"user":{"name":"Ada","email":"ada@example.com"}}"#));

        app.perform_search("ada@example.com");

        assert_eq!(app.search_matches.len(), 1);
        assert_eq!(app.search_match_index, 0);
        assert_eq!(app.response_tree_state.selected().to_vec(), app.search_matches[0]);
    }

    #[test]
    fn search_next_and_prev_cycle_through_matches_with_wraparound() {
        let (mut app, _dir) = test_app();
        app.response = Some(json_response(r#"{"a":"needle","b":"needle"}"#));

        app.perform_search("needle");
        assert_eq!(app.search_matches.len(), 2);
        assert_eq!(app.search_match_index, 0);

        app.jump_to_match(1);
        assert_eq!(app.search_match_index, 1);

        app.jump_to_match(1);
        assert_eq!(app.search_match_index, 0, "should wrap forward past the last match");

        app.jump_to_match(-1);
        assert_eq!(app.search_match_index, 1, "should wrap backward past the first match");
    }

    #[test]
    fn search_with_no_matches_reports_status_and_clears_selection() {
        let (mut app, _dir) = test_app();
        app.response = Some(json_response(r#"{"a":"value"}"#));

        app.perform_search("doesnotexist");

        assert!(app.search_matches.is_empty());
        assert!(app.status_message.as_deref().unwrap_or_default().contains("No matches"));
    }

    #[tokio::test]
    async fn new_response_clears_stale_search_state() {
        let (mut app, _dir) = test_app();
        app.response = Some(json_response(r#"{"a":"needle"}"#));
        app.perform_search("needle");
        assert_eq!(app.search_matches.len(), 1);

        app.handle_app_msg(AppMsg::HttpResult {
            request: Box::new(RequestFile::scratch()),
            result: Ok(json_response(r#"{"b":"other"}"#)),
        });

        assert!(app.search_matches.is_empty(), "a fresh response should drop the previous search");
    }
}
