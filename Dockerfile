# syntax=docker/dockerfile:1.6

############################################
# Base image with Rust + build deps + wasm tools
############################################
FROM rust:1-bookworm AS base
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    pkg-config \
    build-essential \
    clang \
    cmake \
    binaryen \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

ARG WASM_BINDGEN_CLI_VERSION=0.2.108
RUN cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_CLI_VERSION}

############################################
# 1) Server deps (manifest-only cache)
############################################
FROM base AS server_deps
WORKDIR /app/server

COPY server/Cargo.toml server/Cargo.lock ./

# Need a target file present for some cargo operations to be happy.
# (Cheap to copy; good for caching.)
COPY server/src/main.rs ./src/main.rs

RUN cargo fetch

############################################
# 2) Build server binary
############################################
FROM base AS server_build
WORKDIR /app/server

# Reuse cargo registry/cache from deps stage
COPY --from=server_deps /usr/local/cargo /usr/local/cargo

COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src

RUN cargo build --release

############################################
# 3) WASM deps (manifest-only cache)
############################################
FROM base AS wasm_deps
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# Same issue: Cargo wants to see a target
COPY src/main.rs ./src/main.rs

RUN cargo fetch

############################################
# 4) Build wasm + bindgen + wasm-opt + package dist/
############################################
FROM base AS wasm_build
WORKDIR /app

COPY --from=wasm_deps /usr/local/cargo /usr/local/cargo

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY index.html ./index.html

RUN cargo build --target wasm32-unknown-unknown --profile wasm-release

RUN mkdir -p /app/wasm \
    && wasm-bindgen --out-dir /app/wasm --target web \
        /app/target/wasm32-unknown-unknown/wasm-release/valence_sdf.wasm

RUN wasm-opt -Oz \
    --enable-bulk-memory \
    --enable-nontrapping-float-to-int \
    --enable-sign-ext \
    --enable-mutable-globals \
    --strip-debug \
    --strip-producers \
    /app/wasm/valence_sdf_bg.wasm -o /app/wasm/valence_sdf_bg.wasm

RUN rm -rf /app/dist \
    && mkdir -p /app/dist/assets/fonts/fredoka /app/dist/assets/shaders \
    && cp -r /app/wasm/* /app/dist/ \
    && cp /app/index.html /app/dist/ \
    && cp /app/assets/fonts/fredoka/fredoka-bold-digits.json /app/dist/assets/fonts/fredoka/ \
    && cp /app/assets/fonts/fredoka/fredoka-bold-digits.png /app/dist/assets/fonts/fredoka/ \
    && cp /app/assets/shaders/sdf_scene.wgsl /app/dist/assets/shaders/ \
    && cp /app/assets/shaders/seven_segment.wgsl /app/dist/assets/shaders/

############################################
# 5) Runtime: small image with server + dist
############################################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=server_build /app/server/target/release/valence-server /usr/local/bin/valence-server
COPY --from=wasm_build /app/dist /app/dist

ENV PORT=8080
ENV STATIC_DIR=/app/dist
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/valence-server"]
