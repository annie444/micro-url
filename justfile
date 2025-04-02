set unstable
set dotenv-load := true
set dotenv-required := false
set export := true

kanidm_admin_password := "1B0MN71XrDsN7Y4M2ATgKyFCvaXJW2ZcLpLxQP4qG6bTrdyJ"
kanidm_idm_admin_password := "0D6aHBMxWN6JdZRQ78JFqVjPk4GDC20EK6Wf8cPByahQZvcS"

build:
  pnpm exec nx run-many --target=build

lint:
  pnpm exec nx run-many --target=lint

test:
  pnpm exec nx run-many --target=test

run:
  export RUST_LOG=trace
  shuttle run

kanidm-up:
  podman run --detach \
    --name kanidm \
    --publish 8443:8443 \
    --volume ./kanidm:/data:rw \
    docker.io/kanidm/server:latest

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
    --date-time-crate=chrono

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
    podman run --detach \
      --name dev-db \
      --publish 5432:5432 \
      --env POSTGRES_USER=${DB_USER} \
      --env POSTGRES_PASSWORD=${DB_PASS} \
      --env POSTGRES_DB=${DB_NAME} \
      docker.io/library/postgres:latest
  elif command -v docker &> /dev/null; then
    docker run --detach \
      --name dev-db \
      --publish 5432:5432 \
      --env POSTGRES_USER=${DB_USER} \
      --env POSTGRES_PASSWORD=${DB_PASS} \
      --env POSTGRES_DB=${DB_NAME} \
      docker.io/library/postgres:latest
  else
    echo "Unable to find either docker or podman"
  fi

dev-db-stop:
  #!/usr/bin/env bash
  if command -v podman &> /dev/null; then
    podman stop dev-db
    podman rm dev-db
  elif command -v docker &> /dev/null; then
    docker stop dev-db
    docker rm dev-db
  else
    echo "Unable to find either docker or podman"
  fi
