# micro-url

A fast, self-hosted URL shortener with QR codes, click analytics, and OIDC authentication.

[![CI](https://github.com/annie444/micro-url/actions/workflows/ci.yaml/badge.svg)](https://github.com/annie444/micro-url/actions/workflows/ci.yaml) [![CI - Rust](https://github.com/annie444/micro-url/actions/workflows/ci-rust.yaml/badge.svg)](https://github.com/annie444/micro-url/actions/workflows/ci-rust.yaml) [![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

---

## Features

- **URL shortening** — auto-generated or custom slugs, with optional expiry dates
- **QR code generation** — PNG, WebP, or JPEG output with configurable foreground/background colors
- **Click analytics** — per-redirect view tracking with optional IP and HTTP header capture
- **LRU redirect cache** — in-memory cache (1 000 entries) for hot-path redirects, no extra round-trip to the database
- **Authentication** — OIDC federated login (any compliant provider) and local username/password via Argon2
- **Session management** — encrypted private cookies (`axum-extra`), PKCE + CSRF protection on the OIDC flow
- **Background actor pool** — separate Tokio runtime handles periodic session cleanup and expired link purging
- **OpenAPI / Swagger UI** — full interactive docs served at `/api/ui/swagger`
- **Astro + React frontend** — Tailwind CSS, shadcn/ui components, dark-mode support

---

## Quick Start

```sh
# 1. Clone the repository
git clone https://github.com/annie444/micro-url.git && cd micro-url

# 2. Install JS dependencies and build the frontend
pnpm install --frozen-lockfile
pnpm run build

# 3. Build the standalone binary
cargo build --release

# 4. Copy the example config and fill in your values
cp Secrets.example.toml .env   # or use a TOML config file — see Configuration below

# 5. Run
./target/release/micro_url
```

The server starts on `http://127.0.0.1:3000` by default. Open `/ui` for the web interface or `/api/ui/swagger` for interactive API docs.

---

## Self-Hosting

### Prerequisites

| Tool | Minimum version | Notes |
|------|----------------|-------|
| Rust | stable (2024 edition) | `rustup update stable` |
| Node.js | 22.14.0 | Use [asdf](https://asdf-vm.com) or the `.tool-versions` file |
| pnpm | 10.8.0 | `npm install -g pnpm` |
| PostgreSQL | 14+ | Any reachable Postgres instance |

### Building

```sh
# Frontend (required — the binary serves the built assets)
pnpm install --frozen-lockfile
pnpm run build             # outputs to js/frontend/dist/

# Backend binary
cargo build --release      # outputs to target/release/micro_url
```

### Configuration

micro-url reads configuration from environment variables (or a `.env` file via `dotenvy`). You can also pass a TOML config file with `--config <path>`.

#### Server

| Variable | Default | Description |
|----------|---------|-------------|
| `ADDR` | `127.0.0.1` | Bind address |
| `PORT` | `3000` | Bind port |
| `SCHEME` | `http` | URL scheme used when generating short links (`http` or `https`) |
| `INTERNAL_URL` | `{ADDR}:{PORT}` | Internal address the server listens on |
| `EXTERNAL_URL` | `{SCHEME}://{INTERNAL_URL}` | Publicly reachable base URL — used as the prefix for all short links |
| `ASSETS_PATH` | `../../js/frontend/dist` | Path to the built frontend assets directory |

#### Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_USER` | — | PostgreSQL username |
| `DB_PASS` | — | PostgreSQL password |
| `DB_HOST` | `localhost` | PostgreSQL hostname |
| `DB_PORT` | `5432` | PostgreSQL port |
| `DB_NAME` | — | Database name |
| `DB_SCHEMA` | — | Schema name (optional) |

#### OIDC

| Variable | Required | Description |
|----------|----------|-------------|
| `OIDC_CLIENT_ID` | Yes | Client ID from your OIDC provider |
| `OIDC_CLIENT_SECRET` | Yes | Client secret from your OIDC provider |
| `OIDC_DISCOVERY_URL` | Yes | Provider discovery endpoint (e.g. `https://accounts.google.com`) |
| `OIDC_NAME` | No | Display name shown on the login button (default: `default`) |
| `OIDC_SCOPES` | No | Space-separated scopes (default: `openid email profile`) |
| `OIDC_CLAIMS` | No | Space-separated claims to request |
| `OIDC_CERT_PATH` | No | Path to a custom CA certificate for the OIDC provider (PEM or DER) |
| `OIDC_REDIRECT_URL` | — | Must be set in your provider to `{EXTERNAL_URL}/api/user/oidc/callback` |

#### Actor pool (background workers)

| Variable | Default | Description |
|----------|---------|-------------|
| `ACTOR_WORKERS` | `4` | Number of async worker tasks |
| `ACTOR_BLOCKING_WORKERS` | `2` | Number of blocking worker threads |
| `ACTOR_STACK_SIZE` | `2097152` (2 MiB) | Per-thread stack size in bytes |
| `ACTOR_KEEP_ALIVE` | `10s` | Idle thread keep-alive duration |
| `ACTOR_EVENT_INTERVAL` | `61` | Tokio event interval (ticks) |
| `SESSION_CLEAN_INTERVAL` | `10s` | How often expired sessions are purged |
| `SHORT_LINKS_CLEAN_INTERVAL` | `30m` | How often expired short links are purged |

#### IP source (optional analytics)

Enabled by default via the `ips` feature. Set `IP_SOURCE_HEADER` to one of the values accepted by [`axum-client-ip`](https://docs.rs/axum-client-ip), e.g. `RightmostXForwardedFor`, `XRealIp`, or `ConnectInfo` (direct connection).

### Running

**Directly:**

```sh
./target/release/micro_url
# With a custom assets path:
./target/release/micro_url --assets /var/www/micro-url/dist
# With a TOML config file:
./target/release/micro_url --config /etc/micro-url/config.toml
```

**As a systemd service:**

```ini
[Unit]
Description=micro-url
After=network.target postgresql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/micro-url
EnvironmentFile=/opt/micro-url/.env
ExecStart=/opt/micro-url/micro_url --assets /opt/micro-url/dist
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

**Behind nginx (reverse proxy + TLS):**

```nginx
server {
    listen 443 ssl;
    server_name example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Set `EXTERNAL_URL=https://example.com` and `IP_SOURCE_HEADER=XRealIp` (or `RightmostXForwardedFor`) when running behind a proxy.

---

## Development

### Prerequisites

Same as self-hosting, plus:

- **cocogitto** (`cog`) — for conventional commits and changelog generation: `cargo install cocogitto`

### Project structure

```
micro-url/
├── apps/
│   ├── micro_url/       # Binary entry point (CLI flags, config loading)
│   └── server/          # Axum HTTP server library
│       └── src/
│           ├── actor/   # Background task actor pool
│           ├── urls/    # URL shortening routes & structs
│           ├── user/    # User auth routes (local + OIDC)
│           ├── api.rs   # OpenAPI router assembly
│           ├── config.rs
│           ├── state.rs # Shared server state (DB conn, LRU cache, OIDC client)
│           └── ...
├── libs/
│   ├── entity/          # SeaORM entity definitions (auto-generates TS types via ts-rs)
│   └── migration/       # SeaORM database migrations
└── js/
    └── frontend/        # Astro + React frontend
        └── src/
            ├── components/
            ├── lib/api/   # Typed API client
            └── lib/types/ # TypeScript types (generated from Rust structs via ts-rs)
```

> **Note on type safety:** The `ts-rs` crate generates TypeScript type definitions from Rust structs at compile time. Frontend types in `js/frontend/src/lib/types/` are derived from the same structs used by the server — run `cargo build` to regenerate them after changing entity or request/response structs.

### Running in development

```sh
# Terminal 1 — frontend (hot reload)
cd js/frontend && pnpm dev

# Terminal 2 — backend (reads .env or environment)
cargo run --package micro_url
```

### Code conventions

**Commits:** This project uses [Conventional Commits](https://www.conventionalcommits.org/) enforced by `cog`. The `commit-msg` git hook runs `cog verify` automatically after `cog` hooks are installed:

```sh
cog install-hooks
```

**Rust formatting:**

```sh
cargo fmt --all
cargo clippy --all-targets --all-features
```

**JS/TS formatting and linting:**

```sh
pnpm run format   # eslint --fix + prettier --write
pnpm run lint     # eslint + prettier --check
```

**Running all checks (mirrors CI):**

```sh
pnpm exec nx run-many --target=lint
pnpm exec nx run-many --target=build
```

### Database migrations

Migrations are embedded in the binary via `sea-orm-migration` and run automatically on startup. To write a new migration:

```sh
cd libs/migration
cargo run -- generate <migration_name>
```

---

## API Reference

Authentication uses an encrypted session cookie (`sid`) set after login. All routes that require authentication are marked with 🔒.

> Full interactive docs (request/response schemas, try-it-out): **`/api/ui/swagger`**

### URL routes (`/api/url`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/api/url/new` | — | Create a short link. Body: `{ url, short?, expiry?, user? }` |
| `GET` | `/{id}` | — | Redirect to the original URL |
| `GET` | `/api/url/{id}` | — | Get metadata for a short link |
| `PUT` | `/api/url/update/{id}` | — | Update a short link's target URL or slug |
| `DELETE` | `/api/url/delete/{id}` | — | Delete a short link |
| `GET` | `/qr/{id}` | — | Generate a QR code image for a short link |

**QR code query parameters** (`GET /qr/{id}`):

| Parameter | Type | Description |
|-----------|------|-------------|
| `format` | `png` \| `webp` \| `jpeg` | Output image format (default: `png`) |
| `fg_red`, `fg_green`, `fg_blue` | `u8` | Foreground (dark) color — all three required if any is set |
| `fg_alpha` | `u8` | Foreground alpha (default: 255) |
| `bg_red`, `bg_green`, `bg_blue` | `u8` | Background (light) color — all three required if any is set |
| `bg_alpha` | `u8` | Background alpha (default: 255) |

### User routes (`/api/user`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/api/user` | 🔒 | Get the current user's profile |
| `GET` | `/api/user/urls` | 🔒 | Get all short links owned by the current user |
| `GET` | `/api/user/urls/page` | 🔒 | Paginated short links. Params: `page`, `size` |
| `GET` | `/api/user/logout` | 🔒 | Log out and clear the session cookie |

### Local auth (`/api/user/local`)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/user/local/register` | Register a new local account. Body: `{ name, email, password }` |
| `POST` | `/api/user/local/login` | Log in with email and password. Body: `{ email, password }` |

### OIDC auth (`/api/user/oidc`)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/user/oidc/provider` | Returns the configured OIDC provider display name |
| `GET` | `/api/user/oidc/login` | Initiates the OIDC authorization flow (redirects to provider) |
| `GET` | `/api/user/oidc/callback` | OAuth2 callback — exchanges code for session |

### Health

| Method | Path | Description |
|--------|------|-------------|
| `GET` / `HEAD` | `/api/health` | Returns `ok` — use for liveness probes |

---

## Architecture

```
Browser / Client
      │
      ▼
  Axum router  (apps/server/src/api.rs)
      │
      ├── /{id}  ──► LRU cache hit? ──► redirect (no DB)
      │              LRU cache miss? ─► PostgreSQL via SeaORM ──► cache & redirect
      │
      ├── /api/url/*  ──► URL CRUD, QR code generation
      │
      └── /api/user/* ──► Local auth (Argon2) / OIDC (PKCE + CSRF)
                               │
                               ▼
                          Sessions table
                          (PostgreSQL)

Actor pool  (separate Tokio runtime)
  ├── Worker tasks  ──► update view counts on each redirect
  ├── Scheduler     ──► purge expired sessions every ~10s
  └── Scheduler     ──► purge expired short links every ~30m
```

**Short ID generation:** IDs are base-64 encoded from an atomic counter seeded at `100_000_000_000 + (number of existing links)`. The counter uses the character set `[0-9A-Za-z_-]`, producing collision-free, URL-safe slugs that grow in length naturally as the counter increases.

**Type safety across the stack:** The `entity` library uses [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs) to export TypeScript type definitions from the same Rust structs that power the database layer. The frontend's `lib/types/` directory is kept in sync with the backend without any manual type duplication.

**Actor pattern:** The background actor pool runs on its own Tokio runtime (separate from the request-serving runtime), communicating via bounded async channels. This ensures that expensive periodic work — like bulk-deleting expired rows — cannot block request handling.

---

## Contributing

1. Fork the repo and create a feature branch.
2. Install hooks: `cog install-hooks`
3. Make your changes, ensuring all commits follow [Conventional Commits](https://www.conventionalcommits.org/).
4. Run the full check suite before pushing:
   ```sh
   pnpm exec nx run-many --target=lint
   pnpm exec nx run-many --target=build
   cargo fmt --all
   cargo clippy --all-targets --all-features
   ```
5. Open a pull request against `main`.

Changelogs are generated automatically from commit messages by `cog`. See [`CHANGELOG.md`](CHANGELOG.md) for the history.

---

## License

micro-url is licensed under the [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0-or-later).
