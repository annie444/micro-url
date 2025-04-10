set unstable
set dotenv-load := true
set dotenv-required := false
set export := true

default:
  just --list

[doc("Install ALL project dependencies (only needs to be run once)")]
[group("dev")]
install:
  #!/usr/bin/env bash
  # Install rust
  if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
  fi
  # Install just
  if ! command -v just &> /dev/null; then
    cargo install just
  fi
  # Install sea-orm-cli
  if ! command -v sea-orm-cli &> /dev/null; then
    cargo install sea-orm-cli
  fi
  # Install shuttle
  if ! command -v shuttle &> /dev/null; then
    cargo install shuttle
  fi
  # Install git-cliff
  if ! command -v cog &> /dev/null; then
    cargo install cocogitto
  fi
  if ! command -v version &> /dev/null; then
    cargo install version-manager
  fi
  # Install nvm
  if ! command -v npm &> /dev/null; then
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.2/install.sh | bash
    \. "$HOME/.nvm/nvm.sh"
    nvm install
  fi
  # Install pnpm
  if ! command -v pnpm &> /dev/null; then
    curl -fsSL https://get.pnpm.io/install.sh | sh -
  fi
  # Install js dependencies
  if [ -d ./node_modules ]; then
    pnpm install
  fi
  # Ensure a container runtime is available
  if ! command -v podman &> /dev/null && ! command -v docker &> /dev/null; then
    >&2 echo "Unable to find either docker or podman. Please install one of these container runtimes"
    >&2 echo ""
    >&2 echo "Podman is the recommended container runtime. You can install podman from:"
    >&2 echo "https://podman.io/getting-started/installation"
    >&2 echo ""
    >&2 echo "You do not need to re-run this script after installing a container runtime."
  fi

[doc("Runs the NX build sequence")]
[group("nx")]
build:
  pnpm exec nx run-many --target=build

[doc("Runs the NX lint sequence")]
[group("nx")]
lint:
  pnpm exec nx run-many --target=lint

[doc("Runs the NX test sequence")]
[group("nx")]
test:
  pnpm exec nx run-many --target=test

[doc("Alias for `pnpm exec nx'")]
[group("nx")]
nx *args:
  pnpm exec nx {{args}}

[doc("Runs the project in dev mode (`no hot-reloading`)")]
[group("dev")]
run $RUST_LOG="trace": oidc-up (nx "run frontend:build")
  #!/usr/bin/env bash
  set -eo pipefail
  if [ ! -f ./Secrets.toml ]; then
    cp Secrets.example.toml Secrets.toml
  fi
  shuttle run

[doc("Reruns the entity generators (NOTE: you'll need to fix the `#[ts(rename=...)]` decorators manually)")]
[group("db")]
reset-db:
  just dev-db-stop
  just dev-db
  just migrate-up
  just generate
  just dev-db-stop

_container-up name *cmd:
  #!/usr/bin/env bash
  set -exo pipefail
  declare -a container
  container=("--detach" "--name" "{{name}}" {{cmd}})
  if command -v podman &> /dev/null; then
    if ! podman container exists "{{name}}" &> /dev/null; then
      podman run "${container[@]}" 
    fi
  elif command -v docker &> /dev/null; then
    if ! docker container exists "{{name}}" &> /dev/null; then
      docker run "${container[@]}"
    fi
  else
    >&2 echo "Unable to find either docker or podman"
  fi

_container-down name:
  #!/usr/bin/env bash
  set -exo pipefail
  if command -v podman &> /dev/null; then
    if podman container exists "{{name}}" &> /dev/null; then
      podman stop "{{name}}"
      podman rm "{{name}}"
    fi
  elif command -v docker &> /dev/null; then
    if docker container exists "{{name}}" &> /dev/null; then
      docker stop "{{name}}"
      docker rm "{{name}}"
    fi
  else
    >&2 echo "Unable to find either docker or podman"
  fi

[doc("Runs a mock OIDC server in a container (named `oidc`)")]
[group("oidc")]
oidc-up:
  just _container-up "oidc" "--publish" "4011:8080" \
    "--env" "ASPNETCORE_ENVIRONMENT=Development" \
    "--env" "SERVER_OPTIONS_INLINE='{ \
      \"AccessTokenJwtType\": \"JWT\", \
      \"Discovery\": { \
        \"ShowKeySet\": true \
      }, \
      \"Authentication\": { \
        \"CookieSameSiteMode\": \"Lax\", \
        \"CheckSessionCookieSameSiteMode\": \"Lax\" \
      } \
    }'" \
    "--env" "LOGIN_OPTIONS_INLINE='{ \"AllowRememberLogin\": false }'" \
    "--env" "LOGOUT_OPTIONS_INLINE='{ \"AutomaticRedirectAfterSignOut\": true }'" \
    "--env" "CLIENTS_CONFIGURATION_INLINE='[ \
      { \
        \"ClientId\": \"micro-url-mock\", \
        \"ClientSecrets\": [\"micro-url-mock-secret\"], \
        \"Description\": \"Client for authorization code flow\", \
        \"AllowedGrantTypes\": [\"authorization_code\"], \
        \"RequirePkce\": true, \
        \"AllowAccessTokensViaBrowser\": true, \
        \"RedirectUris\": [\"http://localhost:8000/api/user/oidc/callback\"], \
        \"AllowedScopes\": [\"openid\", \"profile\", \"email\"], \
        \"IdentityTokenLifetime\": 3600, \
        \"AccessTokenLifetime\": 3600, \
        \"RequireClientSecret\": false \
      } \
    ]'" \
    "--env" "USERS_CONFIGURATION_INLINE='[ \
      { \
        \"SubjectId\": \"1\", \
        \"Username\": \"User1\", \
        \"Password\": \"password\", \
        \"Claims\": [ \
          { \
            \"Type\": \"name\", \
            \"Value\": \"Test User1\", \
            \"ValueType\": \"string\" \
          }, \
          { \
            \"Type\": \"email\", \
            \"Value\": \"testuser1@example.com\", \
            \"ValueType\": \"string\" \
          }, \
        ] \
      } \
    ]'" \
    "--env" "ASPNET_SERVICES_OPTIONS_INLINE='{ \
      \"ForwardedHeadersOptions\": { \
        \"ForwardedHeaders\": \"All\" \
      } \
    }'" \
    "ghcr.io/soluto/oidc-server-mock:0.9.2"

[doc("Stops and removes the mock OIDC server")]
[group("oidc")]
oidc-down: (_container-down "oidc")

[doc("Runs any pending database migrations")]
[group("db")]
migrate-up:
  sea-orm-cli migrate up \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

[doc("Rollsback all database migrations")]
[group("db")]
migrate-down:
  sea-orm-cli migrate down \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

[doc("Resets all database migrations")]
[group("db")]
migrate-reset:
  sea-orm-cli migrate fresh \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

[doc("Checks for any pending database migrations")]
[group("db")]
migrate-status:
  sea-orm-cli migrate status \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

[doc("Generates the SeaORM entities (`libs/entity`) from the database schema")]
[group("db")]
generate:
  rm -rf entity/src
  sea-orm-cli generate entity \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}" \
    --output-dir "libs/entity/src" \
    --verbose \
    --lib \
    --include-hidden-tables \
    --with-serde=both \
    --serde-skip-hidden-column \
    --date-time-crate=chrono \
    --model-extra-derives "utoipa::ToSchema, ts_rs::TS" \
    --model-extra-attributes "ts(export)" \
    --model-extra-attributes 'ts(export_to = "../../../js/frontend/src/lib/types/")' \
    --enum-extra-derives "utoipa::ToSchema, ts_rs::TS" \
    --enum-extra-attributes "ts(export)" \
    --enum-extra-attributes 'ts(export_to = "../../../js/frontend/src/lib/types/")' \

[doc("Runs a development database in a container (named `dev-db`)")]
[group("db")]
dev-db:
  #!/usr/bin/env bash
  set -eo pipefail
  if [ -z "${DB_PASS+x}" ]; then
    export DB_PASS="postgres"
  fi
  if [ -z "${DB_USER+x}" ]; then
    export DB_USER="postgres"
  fi
  export DB_HOST="127.0.0.1"
  export DB_PORT="5432"
  export DB_NAME="postgres"
  echo 'DB_USER="'${DB_USER}'"' > .env
  echo 'DB_PASS="'${DB_PASS}'"' >> .env
  echo 'DB_HOST="'${DB_HOST}'"' >> .env
  echo 'DB_PORT="'${DB_PORT}'"' >> .env
  echo 'DB_NAME="'${DB_NAME}'"' >> .env
  just _container-up "dev-db" \
    "--publish" "5432:5432" \
    "--env" "POSTGRES_USER=${DB_USER}" \
    "--env" "POSTGRES_PASSWORD=${DB_PASS}" \
    "--env" "POSTGRES_DB=${DB_NAME}" \
    "docker.io/library/postgres:latest"

[doc("Stops the development database container")]
[group("db")]
dev-db-stop: (_container-down "dev-db")
