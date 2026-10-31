#!/bin/sh
# Exercise Docker's actual context filtering with synthetic files only.
set -eu
repository=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
context_root=$(mktemp -d)
trap 'rm -rf "$context_root"' EXIT HUP INT TERM
cp "$repository/.dockerignore" "$context_root/.dockerignore"
for path in Cargo.toml Cargo.lock crates/cube-core/src/lib.rs crates/cube-cli/src/main.rs bindings/python/src/lib.rs bindings/javascript/wasm/Cargo.toml bindings/javascript/wasm/build.rs bindings/javascript/wasm/src/lib.rs bindings/javascript/private.fixture bindings/javascript/node_modules/private.fixture bindings/javascript/wasm/target/private.fixture vendor/salsa20/Cargo.toml vendor/salsa20/README.md vendor/salsa20/src/lib.rs vendor/salsa20/src/backends/soft.rs private-untracked.fixture .env.synthetic crates/cube-core/private.fixture vendor/salsa20/target/private.fixture vendor/salsa20/tests/private.fixture; do
    mkdir -p "$context_root/$(dirname -- "$path")"
    printf 'Synthetic context fixture\n' > "$context_root/$path"
done
docker build --progress=plain --file - "$context_root" <<'DOCKER'
FROM debian:bookworm-slim@sha256:7ff6ba31527b9e336162fa061746c366c4b2032f4d2cff2078bea43f78efe3f8
COPY . /context/
RUN test -f /context/Cargo.toml \
 && test -f /context/Cargo.lock \
 && test -f /context/crates/cube-core/src/lib.rs \
 && test -f /context/crates/cube-cli/src/main.rs \
 && test -f /context/bindings/python/src/lib.rs \
 && test -f /context/bindings/javascript/wasm/Cargo.toml \
 && test -f /context/bindings/javascript/wasm/build.rs \
 && test -f /context/bindings/javascript/wasm/src/lib.rs \
 && test ! -e /context/bindings/javascript/private.fixture \
 && test ! -e /context/bindings/javascript/node_modules/private.fixture \
 && test ! -e /context/bindings/javascript/wasm/target/private.fixture \
 && test -f /context/vendor/salsa20/Cargo.toml \
 && test -f /context/vendor/salsa20/README.md \
 && test -f /context/vendor/salsa20/src/lib.rs \
 && test -f /context/vendor/salsa20/src/backends/soft.rs \
 && test ! -e /context/private-untracked.fixture \
 && test ! -e /context/.env.synthetic \
 && test ! -e /context/crates/cube-core/private.fixture \
 && test ! -e /context/vendor/salsa20/target/private.fixture \
 && test ! -e /context/vendor/salsa20/tests/private.fixture
DOCKER
