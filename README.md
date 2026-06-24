# RustDrop - High-Performance File Sharing

<p align="center">
  <img src="https://raw.githubusercontent.com/UberMetroid/RustDrop/main/frontend/Assets/assets/icon.png" alt="RustDrop Logo" width="128" height="128">
</p>

RustDrop is a lightweight, self-hosted, and high-performance file sharing web application. It features a modern, drag-and-drop web interface for uploading files and folders while maintaining their directory structures, built with a Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## 🐳 Container Installation

### Option 1: Docker Compose (Recommended)

1. Create a `docker-compose.yml` file:

```yaml
version: '3'
services:
  rustdrop:
    image: ubermetroid/rustdrop:latest
    container_name: rustdrop
    restart: unless-stopped
    ports:
      - 4401:4401
    volumes:
      - ./uploads:/app/uploads
    environment:
      - PORT=4401
      - UPLOAD_DIR=/app/uploads
      - BASE_URL=http://localhost:4401/
      - RUSTDROP_TITLE=RustDrop
      - MAX_FILE_SIZE=1024
      - RUSTDROP_PIN=123456
      - AUTO_UPLOAD=true
      - SHOW_FILE_LIST=true
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4401`.

### Option 2: Docker CLI

Run the following command to start the container:

```bash
docker run -d \
  --name rustdrop \
  --restart unless-stopped \
  -p 4401:4401 \
  -v $(pwd)/uploads:/app/uploads \
  -e RUSTDROP_PIN=123456 \
  -e SHOW_FILE_LIST=true \
  ubermetroid/rustdrop:latest
```

---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4401` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. *(Supports fallback `RUSTRUSTDROP_TITLE`)* | `RustDrop` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies to ensure redirect and websocket links are resolved correctly. | `http://localhost:4401/` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `RUSTDROP_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `UPLOAD_DIR` | Main directory path where uploaded files are stored. | `/app/uploads` |
| `MAX_FILE_SIZE` | Maximum file size limit in MB. | `1024` (1GB) |
| `AUTO_UPLOAD` | Start uploading immediately upon dragging files. | `false` |
| `SHOW_FILE_LIST` | Enable file explorer listing/deletion interface. | `false` |
| `TRUST_PROXY` | Set `true` if backend is hosted behind a reverse proxy. | `false` |
| `TRUSTED_PROXY_IPS` | Comma-separated IP list of trusted upstream proxies. | None |
| `MAX_STORAGE_LIMIT_GB` | Maximum capacity limit for upload directory in GB. | None |
| `RETENTION_PERIOD_DAYS` | Automatically delete files older than this many days. | None |
| `ALLOWED_EXTENSIONS` | Comma-separated list of allowed extensions (e.g. `.png,.pdf`). | None (All) |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header (true/false). | `false` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |

## 📂 Repository Structure

```
.
├── backend/
│   ├── Cargo.toml
│   └── src
│       ├── config.rs
│       ├── main.rs
│       ├── routes
│       │   ├── auth.rs
│       │   ├── files
│       │   │   ├── helpers.rs
│       │   │   ├── mod.rs
│       │   │   └── ops.rs
│       │   ├── mod.rs
│       │   └── upload
│       │       ├── cancel.rs
│       │       ├── chunk.rs
│       │       ├── init.rs
│       │       ├── metadata.rs
│       │       ├── mod.rs
│       │       └── utils.rs
│       ├── security.rs
│       ├── tests.rs
│       └── utils.rs
└── frontend/
    ├── Assets
    │   ├── app.css
    │   ├── assets
    │   │   ├── icon.png
    │   │   └── icon.svg
    │   ├── base.css
    │   ├── header.css
    │   ├── login.css
    │   └── service-worker.js
    ├── Cargo.toml
    ├── index.html
    └── src
        ├── api.rs
        ├── app
        │   ├── mod.rs
        │   ├── update_config.rs
        │   ├── update_files.rs
        │   ├── update_pin.rs
        │   ├── update_toast.rs
        │   ├── update_upload.rs
        │   ├── upload_task.rs
        │   └── view
        │       ├── explorer.rs
        │       ├── mod.rs
        │       ├── pin_entry.rs
        │       └── uploader.rs
        ├── header.rs
        ├── i18n
        │   ├── de.rs
        │   ├── en.rs
        │   ├── es.rs
        │   ├── fr.rs
        │   ├── ja.rs
        │   ├── pt.rs
        │   ├── ru.rs
        │   └── zh.rs
        ├── i18n.rs
        ├── js_api.rs
        ├── main.rs
        ├── storage.rs
        ├── types.rs
        └── utils.rs
```
