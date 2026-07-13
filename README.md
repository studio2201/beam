<p align="center">
  <a href="https://github.com/etecoons">
    <img src="assets/header.jpg" alt="etecoons banner" width="100%">
  </a>
</p>

# <img src="assets/icon.png" width="32" height="32" valign="middle"> Beam

[![CI](https://github.com/etecoons/beam/actions/workflows/ci.yml/badge.svg)](https://github.com/etecoons/beam/actions/workflows/ci.yml)

High-performance, secure self-hosted file sharing web application in Rust.

## Quick Start

### Self-Hosting (Docker)
Pull and run the official Docker container:
```bash
docker run -d -p 4401:4401 -v /path/to/appdata:/app/data ghcr.io/etecoons/beam:latest
```
