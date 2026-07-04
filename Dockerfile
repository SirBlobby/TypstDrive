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
    && git -C typst checkout 44b3f78ed37fedea75e911dde2269ef86c45316f
COPY server/Cargo.* server/
COPY server/src server/src
WORKDIR /app/server
RUN cargo build --release

# Final Runtime Image
FROM alpine:3.19
# Provided automatically by buildx (e.g. "amd64", "arm64").
ARG TARGETARCH
WORKDIR /app
RUN apk add --no-cache libgcc openssl pandoc curl sqlite
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
# DB_TYPE         Database backend: "sqlite" or "postgres"
#                 Auto-detected from DATABASE_URL if not set.
# COOKIE_SECRET        64+ byte secret for signing session cookies.
#                      If unset, a random key is generated on each start
#                      and all sessions are invalidated on restart.
# ALLOW_REGISTRATION   Set to "false" to disable public registration.
#                      Admins can still create accounts via the admin panel.
# RUST_LOG             Log filter (default: server=debug,tower_http=debug)
ENV PORT=3000
ENV STATIC_DIR=/app/build
ENV DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
ENV DB_TYPE=sqlite
EXPOSE 3000
CMD ["/app/server"]
