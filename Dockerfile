# Builds the tuner as a static site served by nginx.
#   docker build -t tuner . && docker run --rm -p 8080:80 tuner

# 1. Rust: compile the pitch core to WebAssembly.
FROM rust:1.98-slim AS wasm
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build -p tuner-wasm --target wasm32-unknown-unknown --release

# 2. Node: build the SvelteKit static site around the prebuilt wasm.
FROM node:24-slim AS web
RUN npm install -g pnpm@12.11.2
WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml web/.npmrc ./
RUN pnpm install --frozen-lockfile
COPY web ./
COPY --from=wasm /src/target/wasm32-unknown-unknown/release/tuner_wasm.wasm /tmp/tuner.wasm
ENV TUNER_WASM_PREBUILT=/tmp/tuner.wasm
RUN pnpm build

# 3. nginx: serve the static output.
FROM nginx:1.29-alpine
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=web /src/web/build /usr/share/nginx/html
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s CMD wget -qO- http://127.0.0.1/healthz || exit 1
