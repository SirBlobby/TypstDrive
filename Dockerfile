# Build Frontend
FROM oven/bun:alpine AS frontend-builder
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile
COPY . .
RUN bun run build

# Build Backend
FROM rust:alpine AS backend-builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig git
RUN git clone https://github.com/typst/typst.git typst && cd typst && git checkout d6848a802e86a6269300f9768c054a641c2da77f
COPY server/Cargo.* server/
COPY server/src server/src
WORKDIR /app/server
RUN cargo build --release

# Final Runtime Image
FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache libgcc openssl pandoc curl
RUN curl -L https://github.com/Myriad-Dreamin/tinymist/releases/latest/download/tinymist-alpine-x64 -o /usr/local/bin/tinymist && chmod +x /usr/local/bin/tinymist
COPY --from=frontend-builder /app/build /app/build
COPY --from=backend-builder /app/server/target/release/server /app/server
ENV PORT=3000
ENV STATIC_DIR=/app/build
EXPOSE 3000
CMD ["/app/server"]
