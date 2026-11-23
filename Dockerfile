# Build Frontend
# Frontend output is static, arch-independent assets, so build it natively on the
# build host (no emulation) regardless of the target platform.
FROM --platform=$BUILDPLATFORM oven/bun:alpine AS frontend-builder
WORKDIR /app
COPY package.json bun.lock* ./
RUN bun install
COPY . .
RUN bun run build

# Build Backend
# Built for the target platform (under QEMU emulation for non-native arches).
FROM rust:alpine AS backend-builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig git lld
RUN git clone https://github.com/typst/typst.git typst \
    && git -C typst checkout bd5ebec285505889e8ff1d4f68804ab147a699f3
COPY server/Cargo.* server/
COPY server/.cargo server/.cargo
WORKDIR /app/server
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY server/src src
RUN touch src/main.rs && cargo build --release && cp target/release/server /app/server-binary

# Final Runtime Image
FROM alpine:3.19
# Provided automatically by buildx (e.g. "amd64", "arm64").
ARG TARGETARCH
WORKDIR /app
RUN apk add --no-cache libgcc openssl pandoc curl sqlite font-noto-emoji git ca-certificates
RUN mkdir -p /data
# Install tinymist for the target architecture.
ARG TINYMIST_VERSION=v0.15.12
RUN case "$TARGETARCH" in \
        amd64) TINYMIST_TRIPLE="x86_64-unknown-linux-musl" ;; \
        arm64) TINYMIST_TRIPLE="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    curl -fL "https://github.com/Myriad-Dreamin/tinymist/releases/download/${TINYMIST_VERSION}/tinymist-${TINYMIST_TRIPLE}.tar.gz" -o /tmp/tinymist.tar.gz && \
    tar -xzf /tmp/tinymist.tar.gz -C /usr/local/bin --strip-components=1 "tinymist-${TINYMIST_TRIPLE}/tinymist" && \
    chmod +x /usr/local/bin/tinymist && \
    rm /tmp/tinymist.tar.gz
# Install tectonic for LaTeX documents. The static musl build is required because
# LaTeX compiles run inside a chroot jail. Checksums pin the exact release files.
ARG TECTONIC_VERSION=0.17.0
RUN case "$TARGETARCH" in \
        amd64) TECTONIC_TRIPLE="x86_64-unknown-linux-musl"; TECTONIC_SHA256="8533d07f9ccbd7a65824b9e0459041bca34af1eb33daba48f59215593753a3b7" ;; \
        arm64) TECTONIC_TRIPLE="aarch64-unknown-linux-musl"; TECTONIC_SHA256="b10954a95404f3ab2328d2fa59a5ebab8e657f893fab096f98be8db7c0c979b8" ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    curl -fL "https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%40${TECTONIC_VERSION}/tectonic-${TECTONIC_VERSION}-${TECTONIC_TRIPLE}.tar.gz" -o /tmp/tectonic.tar.gz && \
    echo "${TECTONIC_SHA256}  /tmp/tectonic.tar.gz" | sha256sum -c - && \
    tar -xzf /tmp/tectonic.tar.gz -C /usr/local/bin tectonic && \
    chmod 755 /usr/local/bin/tectonic && \
    rm /tmp/tectonic.tar.gz
COPY --from=frontend-builder /app/build /app/build
COPY --from=backend-builder /app/server-binary /app/server

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
# PUBLIC_URL           Address people use to open TypstDrive. Needed for single sign-on and email links.
# TRUST_PROXY_HEADERS  Set to "true" behind a reverse proxy to read X-Forwarded-For
# GIT_CACHE_DIR        Disposable partial clones for Git servers other than GitHub and GitLab
# SMTP_PROVIDER        "google" fills in the Gmail server and port
# SMTP_HOST, SMTP_PORT, SMTP_SECURITY, SMTP_USERNAME, SMTP_PASSWORD, SMTP_FROM
#                      Outgoing mail for sign-in codes, verification, password reset and invitations
# OIDC_PROVIDERS       Comma-separated OpenID Connect provider IDs, each configured with
#                      OIDC_<ID>_ISSUER, OIDC_<ID>_CLIENT_ID and OIDC_<ID>_CLIENT_SECRET
# GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_HOSTED_DOMAIN   Sign in with Google
# GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET                         Sign in with GitHub
# LATEX_DIR            Sandbox and package cache for LaTeX compiles (default here: /data/latex)
# LATEX_TIMEOUT_SECONDS, LATEX_MAX_PARALLEL, LATEX_MEMORY_MB     Limits for each LaTeX compile
# LATEX_BUNDLE         URL or path of a tectonic bundle to use instead of the default one
ENV PORT=3000
ENV STATIC_DIR=/app/build
ENV DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
ENV GIT_CACHE_DIR=/data/git-cache
ENV LATEX_DIR=/data/latex
EXPOSE 3000
CMD ["/app/server"]
