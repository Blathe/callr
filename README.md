# callr

A terminal HTTP client — a Postman-style workflow that never leaves the terminal. Compose requests, send them asynchronously, and browse the response as a collapsible, searchable tree, all with vim-style modal keybindings.

Built with [Ratatui](https://ratatui.rs), [Tokio](https://tokio.rs), and [reqwest](https://github.com/seanmonstar/reqwest).

## Features

- **In-app request editor** — URL, method, headers, query params, and body, edited directly in the TUI.
- **Git-friendly collections** — requests are plain TOML files on disk, one file per request; folders are the collection hierarchy. Diff them, review them, commit them.
- **Local request history** — every send is snapshotted to a SQLite database (`.callr/history.db`), independent of the live collection files, so editing or deleting a request never touches its history. History entries are searchable and re-runnable.
- **Built-in auth helpers** — Bearer, Basic, and API key (as a header or query param).
- **Collapsible response viewer** — JSON and XML responses render as a syntax-highlighted, expandable tree, with in-body search (`/`, `n`/`N`). Binary/non-UTF8 bodies show a byte-count placeholder instead of garbling the screen.
- **Non-blocking sends** — requests run on a Tokio task and report back over a channel, so the UI stays responsive while a request is in flight.
- **Vim-style modal keybindings** — Normal/Insert/Command/Search modes, `hjkl` navigation, `gg`/`G` jump-to-top/bottom, `:` command line.

Environment variables / `{{var}}` substitution are intentionally out of scope for now — the data model is substitution-friendly (plain strings throughout) so this can be added later without a schema change.

## Installation

Requires a recent stable Rust toolchain ([rustup.rs](https://rustup.rs)).

```sh
git clone <this-repo>
cd callr
cargo build --release
```

The binary is written to `target/release/callr` (`callr.exe` on Windows).

## Usage

```sh
callr                    # launch in the current directory
callr <collection-path>  # launch pointed at a specific collection directory
```

On launch, `callr` walks the given directory (or cwd) for `.toml` request files and folders and renders them as a collection tree in the sidebar. A `.callr/` directory is created alongside it on first send, holding the SQLite history database — it's excluded from the collection tree automatically.

## Keybindings

Normal mode is the default; press `Esc` from Insert/Command/Search to return to it.

| Key | Action |
|---|---|
| `Tab` | Cycle focus between panes |
| `i` | Enter Insert mode on the focused field |
| `:` | Enter Command mode |
| `q` | Quit |
| `j`/`k` (or `↓`/`↑`) | Move selection / scroll |
| `h`/`l` (or `←`/`→`) | Collapse / expand tree node |
| `gg` / `G` | Jump to top / bottom |
| `Enter` | Send request (URL bar) · open node (sidebar/response tree) |

**Auth pane** (`Tab` to focus it): `a` cycles the auth type (None → Bearer → Basic → API Key), `j`/`k` moves between fields, `i` edits the focused field, `l`/`Enter` toggles header/query placement for API keys.

**Response pane**: `/` starts an in-body search, `n`/`N` jump to the next/previous match (auto-expanding collapsed ancestors).

### Command mode (`:`)

| Command | Effect |
|---|---|
| `:w` | Save the current request to its TOML file |
| `:new <name>` | Create a new scratch request |
| `:history` | Switch the sidebar to request history |
| `:collections` | Switch the sidebar back to the collection tree |
| `:find <query>` | Search request history |
| `:q` | Quit |
| `:env` | Reserved — environment variables aren't implemented yet |

## Collection format

Each request is a single TOML file. Folders nest to form the collection hierarchy — there's no separate manifest.

```toml
method = "GET"
url = "https://httpbin.org/get"

[[headers]]
name = "Accept"
value = "application/json"
enabled = true

[meta]
id = "b3f1c2e4-1a2b-4c3d-9e8f-0011223344ab"
name = "Get User"

[auth]
type = "none"

[body]
type = "none"
```

> **Note:** TOML requires scalar fields (`url`, `method`, ...) to appear *before* any table headers (`[meta]`, `[auth]`, `[body]`) in the file — anything after a table header is parsed as belonging to that table. `callr` always serializes files in the correct order; if you hand-edit a file, keep scalars above the first `[table]`.

Sample requests live under [`collections/`](collections) if you want to try the app against `httpbin.org` without setting anything up.

## Development

```sh
cargo build              # debug build
cargo test                # unit + snapshot tests
cargo test -- --ignored    # live tests against httpbin.org (needs network)
cargo clippy --all-targets  # lint
```

Snapshot tests use [`insta`](https://insta.rs) against a `ratatui::TestBackend` render buffer — run `cargo insta review` after an intentional UI change to accept updated snapshots.

### Notes for local HTTPS development

`callr` uses `reqwest`'s `rustls-tls-native-roots` backend, which trusts whatever your OS certificate store trusts (e.g. a cert installed via `dotnet dev-certs https --trust` on Windows), rather than only the bundled Mozilla CA list. If you hit a `client error (Connect): invalid peer certificate` error against a local dev server, make sure its certificate is trusted by the OS, not just by curl/your browser.

## Project layout

```
src/
  main.rs        entry point, terminal init/teardown, event loop
  app.rs          central App state, dispatch, and command handling
  mode.rs          Mode/Focus/SidebarView enums
  keymap.rs         key -> Action resolution
  action.rs          Action enum
  model/               request/response/auth/history/collection data types
  http/                 reqwest request building + async send task
  storage/               TOML load/save, collection directory walking
  db/                      SQLite connection, schema, history repository
  ui/                       Ratatui render functions, one module per pane
```

## License

No license has been chosen for this project yet.
