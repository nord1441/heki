# Stage 1: Build frontend
FROM node:20-slim AS frontend-build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html vite.config.ts tsconfig.json tsconfig.node.json ./
COPY public/ public/
COPY src/ src/
RUN npm run build

# Stage 2: Build Tauri application (Linux)
FROM rust:1.82-bookworm AS tauri-build

RUN apt-get update && apt-get install -y --no-install-recommends \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    libasound2-dev \
    pkg-config \
    curl \
    wget \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
COPY --from=frontend-build /app/dist ./dist
RUN cd src-tauri && cargo build --release
RUN cd src-tauri && cargo install tauri-cli --version "^2" \
    && cargo tauri build --bundles deb,appimage 2>/dev/null || true

# Stage 3: Web frontend served via nginx
FROM nginx:1.27-alpine AS web
COPY --from=frontend-build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
