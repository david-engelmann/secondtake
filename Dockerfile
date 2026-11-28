# syntax=docker/dockerfile:1
FROM rust:1.82-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/secondtake /usr/local/bin/secondtake
COPY LICENSE README.md /usr/local/share/doc/secondtake/
WORKDIR /work
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/secondtake"]
CMD ["--help"]
