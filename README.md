# Beam — High-Performance File Sharing <img src="https://raw.githubusercontent.com/UberMetroid/unraid-templates/main/icons/beam.png" width="48" height="48" alt="beam logo" align="right">

Beam is a lightweight, self-hosted, and high-performance file sharing web application. It features a modern, drag-and-drop web interface for uploading files and folders while maintaining their directory structures. Built with a high-performance Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## 🏛️ Architecture & Stack
*   **Frontend**: Yew (WASM)
*   **Backend**: Axum (Rust) / Tokio
*   **Deployment**: Nix-built Container / Unraid native / Docker Compose

---

## 🟢 Key Features
*   **Drag-and-Drop Uploads**: Upload files and complete folder structures seamlessly while preserving directory layouts.
*   **Access PIN Security**: Lock down the interface with an optional numerical PIN for absolute privacy.
*   **Quota & Retention**: Configurable total storage limits and automatic age-based file purges.
*   **Dynamic Themes**: Super Metroid UI themes (Crateria, Brinstar, Norfair, Wrecked Ship, Maridia, Tourian).
*   **Internationalization**: Built-in multilingual translation selector support.
*   **Print Optimization**: Customized print stylesheet layout and print header action button.
*   **Performance First**: Tiny resource footprint, zero external JS engine dependencies, and rapid page load speeds.

---

## 💾 Deployment & Installation

### Docker Compose
Create a `docker-compose.yml` file with the following service definition:

```yaml
services:
  beam:
    image: ubermetroid/beam:latest
    container_name: beam
    restart: unless-stopped
    ports:
      - ${PORT:-4401}:4401
    volumes:
      - ${BEAM_UPLOADS_PATH:-./uploads}:/app/uploads
      - ${BEAM_DATA_PATH:-./data}:/app/data
    environment:
      PORT: 4401
      SITE_TITLE: ${BEAM_SITE_TITLE:-Beam}
      BEAM_PIN: ${BEAM_PIN:-}
      BASE_URL: ${BEAM_BASE_URL:-http://localhost:4401}
      ALLOWED_ORIGINS: ${BEAM_ALLOWED_ORIGINS:-*}
      TZ: ${TZ:-UTC}
      MAX_FILE_SIZE: 1024
      AUTO_UPLOAD: "true"
      SHOW_FILE_LIST: "true"
      UPLOAD_DIR: /app/uploads
      ENABLE_TRANSLATION: ${ENABLE_TRANSLATION:-false}
      ENABLE_THEMES: ${ENABLE_THEMES:-true}
      ENABLE_PRINT: ${ENABLE_PRINT:-true}
      MAX_ATTEMPTS: ${MAX_ATTEMPTS:-5}
```

---

## ⚙️ Configuration Options

| Environment Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4401` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. | `Beam` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies. | `http://localhost:4401/` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). | `*` |
| `BEAM_PIN` | Optional 4–64 character PIN to lock access to the interface. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `UPLOAD_DIR` | Main directory path where uploaded files are stored. | `/app/uploads` |
| `MAX_FILE_SIZE` | Maximum file size limit in MB. | `1024` (1GB) |
| `AUTO_UPLOAD` | Start uploading immediately upon dragging files. | `false` |
| `SHOW_FILE_LIST` | Enable file explorer listing/deletion interface. | `true` |
| `TRUST_PROXY` | Set `true` if backend is hosted behind a reverse proxy. | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated IP/CIDR list of trusted upstream proxies. | None |
| `MAX_STORAGE_LIMIT_GB` | Maximum capacity limit for upload directory in GB. | None |
| `RETENTION_PERIOD_DAYS` | Automatically delete files older than this many days. | None |
| `ALLOWED_EXTENSIONS` | Comma-separated list of allowed extensions (e.g. `.png,.pdf`). | None (All) |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header. | `false` |
| `ENABLE_THEMES` | Enable the Super Metroid theme selector in the navigation header. | `true` |
| `ENABLE_PRINT` | Enable the print button in the navigation header. | `true` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before lockout. | `5` |
| `LOCKOUT_TIME_MINUTES` | Lockout duration in minutes for IPs exceeding `MAX_ATTEMPTS`. | `15` |
| `COOKIE_MAX_AGE_HOURS` | Duration in hours that the user's PIN session cookie remains valid. | `24` |
| `SHUTDOWN_DRAIN_SECONDS` | Seconds to wait for active connections to finish before shutting down. | `5` |
| `SHOW_VERSION` | Display the application version number in the footer. | `true` |
| `SHOW_GITHUB` | Display the GitHub repository link in the footer. | `true` |
| `CLIENT_MAX_RETRIES` | Number of connection retry attempts permitted for chunked file uploads. | `5` |

---

## 🛠️ Local Development

Ensure you have the Rust toolchain and Trunk installed.

```bash
# 1. Run workspace tests
cargo test

# 2. Run clippy workspace checks
cargo clippy --workspace --all-targets

# 3. Start frontend Yew dev server (from frontend/)
cd frontend && trunk serve

# 4. Start backend Axum server (from backend/)
cd backend && cargo run
```

---

## 📄 License
Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 UberMetroid.
