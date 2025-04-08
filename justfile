set unstable
set dotenv-load := true
set dotenv-required := false
set export := true

kanidm_admin_password := "1B0MN71XrDsN7Y4M2ATgKyFCvaXJW2ZcLpLxQP4qG6bTrdyJ"
kanidm_idm_admin_password := "0D6aHBMxWN6JdZRQ78JFqVjPk4GDC20EK6Wf8cPByahQZvcS"

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
  # Install kanidm
  if ! command -v kanidm &> /dev/null; then
    cargo install kanidm_tools
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

build:
  pnpm exec nx run-many --target=build

lint:
  pnpm exec nx run-many --target=lint

test:
  pnpm exec nx run-many --target=test

run $RUST_LOG="trace": kanidm-up build
  #!/usr/bin/env bash
  set -eo pipefail
  if [ ! -f ./Secrets.toml ]; then
    cp Secrets.example.toml Secrets.toml
  fi
  shuttle run

reset-db:
  just dev-db-stop
  just dev-db
  just migrate-up
  just generate
  just dev-db-stop

kanidm-login:
  kanidm login --username idm_admin --password "{{kanidm_idm_admin_password}}"

kanidm *args:
  kanidm {{args}}

kanidm-up:
  #!/usr/bin/env bash
  set -eo pipefail
  if command -v podman &> /dev/null; then
    if ! podman container exists kanidm &> /dev/null; then
      podman run --detach \
        --name kanidm \
        --publish 8443:8443 \
        --volume ./kanidm:/data:rw \
        docker.io/kanidm/server:latest
    fi
  elif command -v docker &> /dev/null; then
    if ! docker container exists kanidm &> /dev/null; then
      docker run --detach \
        --name kanidm \
        --publish 8443:8443 \
        --volume ./kanidm:/data:rw \
        docker.io/kanidm/server:latest
    fi
  else
    >&2 echo "Unable to find either docker or podman"
  fi

kanidm-down:
  #!/usr/bin/env bash
  set -eo pipefail
  if command -v podman &> /dev/null; then
    if podman container exists kanidm &> /dev/null; then
      podman stop kanidm
      podman rm kanidm
    fi
  elif command -v docker &> /dev/null; then
    if docker container exists kanidm &> /dev/null; then
      docker stop kanidm
      docker rm kanidm
    fi
  else
    >&2 echo "Unable to find either docker or podman"
  fi

migrate-up:
  sea-orm-cli migrate up \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

migrate-down:
  sea-orm-cli migrate down \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

migrate-reset:
  sea-orm-cli migrate fresh \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

migrate-status:
  sea-orm-cli migrate status \
    --migration-dir ./libs/migration/ \
    --database-url "postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

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
  if command -v podman &> /dev/null; then
    if ! podman container exists dev-db &> /dev/null; then
      podman run --detach \
        --name dev-db \
        --publish 5432:5432 \
        --env POSTGRES_USER=${DB_USER} \
        --env POSTGRES_PASSWORD=${DB_PASS} \
        --env POSTGRES_DB=${DB_NAME} \
        docker.io/library/postgres:latest
    fi
  elif command -v docker &> /dev/null; then
    if ! docker container exists dev-db &> /dev/null; then
      docker run --detach \
        --name dev-db \
        --publish 5432:5432 \
        --env POSTGRES_USER=${DB_USER} \
        --env POSTGRES_PASSWORD=${DB_PASS} \
        --env POSTGRES_DB=${DB_NAME} \
        docker.io/library/postgres:latest
    fi
  else
    >&2 echo "Unable to find either docker or podman"
  fi

dev-db-stop:
  #!/usr/bin/env bash
  set -eo pipefail
  if command -v podman &> /dev/null; then
    if podman container exists dev-db &> /dev/null; then
      podman stop dev-db
      podman rm dev-db
    fi
  elif command -v docker &> /dev/null; then
    if docker container exists dev-db &> /dev/null; then
      docker stop dev-db
      docker rm dev-db
    fi
  else
    >&2 echo "Unable to find either docker or podman"
  fi
