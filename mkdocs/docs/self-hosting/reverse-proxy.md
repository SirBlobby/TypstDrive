# HTTPS and reverse proxies

To reach TypstDrive safely from the internet, put a **reverse proxy** in front of it. The proxy answers on your domain with HTTPS and passes the traffic on to TypstDrive.

## Before you start

1. Point your domain, such as `typst.example.com`, at your server's IP address with an `A` record at your DNS provider.
2. Make sure ports 80 and 443 are open on the server.
3. Set these in `docker-compose.yml` and restart TypstDrive:

    ```yaml
    environment:
      - PUBLIC_URL=https://typst.example.com
      - TRUST_PROXY_HEADERS=true
    ```

    `PUBLIC_URL` must match the address people use. `TRUST_PROXY_HEADERS` lets TypstDrive see each visitor's real address, which it needs for sign-in protection. Only turn it on behind a proxy.

TypstDrive uses WebSockets for live editing, so the proxy must pass those through. Both examples below do.

## Caddy (easiest)

[Caddy](https://caddyserver.com/) gets and renews HTTPS certificates for you. Add it to the same `docker-compose.yml`:

```yaml
services:
  typstdrive:
    image: ghcr.io/sirblobby/typstdrive:latest
    restart: unless-stopped
    environment:
      - COOKIE_SECRET=paste-your-secret-here
      - PUBLIC_URL=https://typst.example.com
      - TRUST_PROXY_HEADERS=true
      - RUST_LOG=info
    volumes:
      - ./data:/data

  caddy:
    image: caddy:2
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - ./caddy:/data
```

TypstDrive no longer needs its own `ports` entry, because only Caddy talks to it. Then create a file named `Caddyfile` next to it:

```text
typst.example.com {
    reverse_proxy typstdrive:3000
}
```

Run `docker compose up -d`. After a few seconds, `https://typst.example.com` works.

## nginx

If you already run nginx, add a server block like this. Certificates can come from [Certbot](https://certbot.eff.org/).

```nginx
server {
    listen 443 ssl;
    server_name typst.example.com;

    ssl_certificate     /etc/letsencrypt/live/typst.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/typst.example.com/privkey.pem;

    client_max_body_size 100m;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 1h;
    }
}
```

`client_max_body_size` allows larger uploads, and `proxy_read_timeout` keeps live editing connections open.

## Other proxies

Traefik, Nginx Proxy Manager, Cloudflare Tunnel and others work too. Whatever you use, make sure it:

- forwards WebSocket connections;
- allows uploads of the size you need;
- sends the `X-Forwarded-For` or `X-Real-IP` header.

## Checking it works

- Open your domain. You should see the TypstDrive sign-in page with a padlock in the address bar.
- Open a document in two browsers. Typing in one should show up in the other straight away. If not, WebSockets are not getting through.
- **Site Settings → Authentication** warns you if `PUBLIC_URL` is missing.
