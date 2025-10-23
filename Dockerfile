# ---------- build stage ----------
FROM rust:slim-bookworm AS builder

RUN apt-get update
RUN apt-get install -y --no-install-recommends \
    ca-certificates curl unzip git pkg-config libssl-dev unzip clang

RUN curl -sSL -o /usr/local/bin/buf https://github.com/bufbuild/buf/releases/download/v1.59.0/buf-Linux-x86_64
RUN chmod +x /usr/local/bin/buf

RUN curl -sSL -o protoc-33.0-linux-x86_64.zip https://github.com/protocolbuffers/protobuf/releases/download/v33.0/protoc-33.0-linux-x86_64.zip
RUN unzip protoc-33.0-linux-x86_64.zip
RUN chmod +x bin/protoc
RUN mv bin/protoc /usr/local/bin
RUN mv include /usr/local

WORKDIR /app
COPY . .
RUN cargo build --release

# ---------- runtime stage (optional, for a tiny image) ----------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/light-certificate-validator /usr/local/bin/light-certificate-validator

EXPOSE 50051
ENTRYPOINT ["/usr/local/bin/light-certificate-validator"]
