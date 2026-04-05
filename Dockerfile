# Build Frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app
COPY package*.json ./
RUN npm i
COPY . .
RUN npm run build

# Build Backend
FROM rust:alpine AS backend-builder
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig git
RUN git clone -b v0.14.2 --single-branch https://github.com/typst/typst.git typst
COPY server/Cargo.* server/
COPY server/src server/src
WORKDIR /app/server
RUN cargo build --release

# Final Runtime Image
FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache libgcc openssl pandoc
COPY --from=frontend-builder /app/build /app/build
COPY --from=backend-builder /app/server/target/release/server /app/server
ENV PORT=3000
ENV STATIC_DIR=/app/build
EXPOSE 3000
CMD ["/app/server"]
