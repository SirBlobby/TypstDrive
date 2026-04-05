# Build Frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Build Backend
FROM rust:1.82-alpine AS backend-builder
WORKDIR /app
RUN apk add --no-cache musl-dev sqlite-dev openssl-dev pkgconfig
COPY typst/ typst/
COPY server/Cargo.* server/
COPY server/src server/src
WORKDIR /app/server
RUN cargo build --release

# Final Runtime Image
FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache libgcc sqlite-libs openssl
COPY --from=frontend-builder /app/build /app/build
COPY --from=backend-builder /app/server/target/release/server /app/server
ENV PORT=3000
EXPOSE 3000
CMD ["/app/server"]
