# Build Frontend
# Frontend output is static, arch-independent assets, so build it natively on the
# build host (no emulation) regardless of the target platform.
FROM --platform=$BUILDPLATFORM oven/bun:alpine AS frontend-builder
WORKDIR /app
COPY package.json ./
RUN bun install
COPY . .
RUN bun run build

# Build Backend
# Built for the target platform (under QEMU emulation for non-native arches).
FROM rust:alpine AS backend-builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig git
RUN git clone https://github.com/typst/typst.git typst \
    && git -C typst checkout bd5ebec285505889e8ff1d4f68804ab147a699f3
COPY server/Cargo.* server/
COPY server/src server/src
WORKDIR /app/server
RUN cargo build --release

# Final Runtime Image
FROM alpine:3.19
# Provided automatically by buildx (e.g. "amd64", "arm64").
ARG TARGETARCH
WORKDIR /app
RUN apk add --no-cache libgcc openssl pandoc curl sqlite font-noto-emoji
RUN mkdir -p /data
# Install tinymist for the target architecture.
RUN case "$TARGETARCH" in \
        amd64) TINYMIST_TRIPLE="x86_64-unknown-linux-musl" ;; \
        arm64) TINYMIST_TRIPLE="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    curl -fL "https://github.com/Myriad-Dreamin/tinymist/releases/latest/download/tinymist-${TINYMIST_TRIPLE}.tar.gz" -o /tmp/tinymist.tar.gz && \
    tar -xzf /tmp/tinymist.tar.gz -C /usr/local/bin --strip-components=1 "tinymist-${TINYMIST_TRIPLE}/tinymist" && \
    chmod +x /usr/local/bin/tinymist && \
    rm /tmp/tinymist.tar.gz
COPY --from=frontend-builder /app/build /app/build
COPY --from=backend-builder /app/server/target/release/server /app/server

# --- Environment Variables ---
# PORT            Server listen port (default: 3000)
# STATIC_DIR      Path to compiled frontend assets (default: /app/build)
# DATABASE_URL    Database connection URL
#                   SQLite:   sqlite:///data/typstdrive.db?mode=rwc
#                   Postgres: postgres://user:pass@host:5432/typstdrive
#                   MySQL:    mysql://user:pass@host:3306/typstdrive
# DB_TYPE         Database backend: "sqlite", "postgres", "mysql" or "mariadb"
#                 Auto-detected from DATABASE_URL if not set.
# COOKIE_SECRET        64+ byte secret for signing session cookies.
#                      If unset, a random key is generated on each start
#                      and all sessions are invalidated on restart.
# ALLOW_REGISTRATION   Set to "false" to disable public registration.
#                      Admins can still create accounts via the admin panel.
# RUST_LOG             Log filter (default: server=debug,tower_http=debug)
# REDIS_URL            Redis or Valkey URL for real-time collaboration across instances.
# REDIS_CHANNEL_PREFIX Prefix for the pub/sub channels (default: typstdrive)
# S3_BUCKET            Bucket for uploaded files. Turns on object storage (RustFS, AWS S3, R2).
# S3_ENDPOINT          Endpoint for S3-compatible services, for example http://rustfs:9000
# S3_REGION            Bucket region (default: us-east-1)
# S3_ACCESS_KEY_ID     Access key for the bucket
# S3_SECRET_ACCESS_KEY Secret key for the bucket
# S3_PREFIX            Folder inside the bucket (default: typstdrive)
# S3_VIRTUAL_HOSTED_STYLE  Set to "true" for bucket.host style URLs
# OBJECT_CACHE_MB      Memory cache for files read from object storage (default: 256)
ENV PORT=3000
ENV STATIC_DIR=/app/build
ENV DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
EXPOSE 3000
CMD ["/app/server"]
