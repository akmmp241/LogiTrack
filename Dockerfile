# why use debian slim bookworm? simply bcs it small and but enough for this dev case
FROM rust:1.91.1-slim-bookworm AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


# idrk what cargo chef is, but i'm pretty sure this gonna cache all build dependencies so it gonna be build so much
# faster than the first time
FROM rust:1.91.1-slim-bookworm AS cacher
WORKDIR /app
RUN cargo install cargo-chef

# mold is like a faster and more modern linker for build,
# thanks to my frind for referencing this
RUN apt-get update && apt-get install -y mold clang pkg-config libssl-dev

ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --recipe-path recipe.json


FROM rust:1.91.1-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y mold clang pkg-config libssl-dev
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

COPY . .

ARG BINARY_NAME
RUN cargo build --bin ${BINARY_NAME}


FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

ARG BINARY_NAME
COPY --from=builder /app/target/debug/${BINARY_NAME} /app/server

EXPOSE 3000

CMD ["./server"]