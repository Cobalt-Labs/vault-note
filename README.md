# 📝 VaultNote

> A secure, blazing-fast fullstack web application for capturing and managing notes, built with **Rust (WASM + Leptos)** on the frontend and **Axum** on the backend.

---

## 🌐 Overview

**VaultNote** is a web-targeted personal vault application designed for fast, distraction-free note taking with instant persistence and keyboard-driven workflows. The application leverages WebAssembly in the browser for near-native client performance and an asynchronous Rust API backend.

Planned for upcoming releases is a complete identity and session management layer backed by **Firebase Authentication**, supporting both **Google Sign-In** and **Phone Number (SMS OTP)** verification.

---

## 🛠️ Tech Stack

### Frontend
- **Framework:** [Leptos](https://leptos.dev/) `v0.8` (Client-Side Rendering / WASM)
- **Tooling & Bundler:** [Trunk](https://trunkrs.dev/) `v0.22`
- **Styling:** [Tailwind CSS](https://tailwindcss.com/) `v4`
- **Utilities:** `leptos-use` (keyboard listeners), `gloo-net` (HTTP client), `uuid`

### Backend
- **Framework:** [Axum](https://github.com/tokio-rs/axum) `v0.8`
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **Serialization:** [Serde](https://serde.rs/) (JSON)

### Authentication & Cloud (Planned)
- **Identity Provider:** [Firebase Authentication](https://firebase.google.com/docs/auth)
  - Google OAuth 2.0
  - Phone Number Authentication (SMS OTP)

---

## 📋 Project Roadmap & Completion Checklist

Track the progress of VaultNote from initial prototype to production web deployment:

### Phase 1: Core Foundation & Layout
- [x] Rust workspace configuration (`frontend` + `backend`)
- [x] Leptos WASM frontend setup with Trunk
- [x] Tailwind CSS v4 styling pipeline and pre-build automation
- [x] Basic note drafting interface (Title, Content, Status indicator)
- [x] Keyboard shortcuts (`Ctrl + S` quick save)
- [x] Axum backend setup with JSON endpoints (`POST /api/notes`)

### Phase 2: Firebase Authentication (Next Step)
- [ ] Firebase project setup and web client SDK integration
- [ ] Google OAuth sign-in flow
- [ ] Phone number authentication with SMS OTP verification
- [ ] Auth state listener & reactive user session context in Leptos
- [ ] Backend JWT / Firebase ID token verification middleware

### Phase 3: Vault Note Management
- [ ] Note list sidebar with real-time sync
- [ ] Note editing, renaming, and soft-delete capabilities
- [ ] Markdown preview & rich-text formatting
- [ ] Persistent storage (Database integration on backend: SQLite / PostgreSQL)
- [ ] Search and tag-based filtering

### Phase 4: Polish & Deployment
- [ ] Dark / Light mode theme toggle
- [ ] Auto-save with debounced drafts
- [ ] Offline caching and PWA support
- [ ] Production build optimization & Docker containerization
- [ ] Cloud deployment (Vercel / Cloudflare / Fly.io / GCP)

---

## 📂 Project Structure

```text
vault-note/
├── backend/                  # Rust Axum API server
│   ├── src/
│   │   ├── handlers/         # Request handlers (save_note, etc.)
│   │   ├── models.rs         # Shared data models
│   │   ├── routes.rs         # API routing configuration
│   │   └── main.rs           # Server entry point (binds to 127.0.0.1:3000)
│   └── Cargo.toml
├── frontend/                 # Leptos WASM web application
│   ├── src/
│   │   ├── app.rs            # Main UI component & state logic
│   │   ├── models.rs         # Note data models
│   │   └── main.rs           # Web client entry point
│   ├── style/
│   │   └── tailwind.css      # Tailwind v4 directives & sources
│   ├── index.html            # HTML shell with data-trunk asset hooks
│   ├── package.json          # Node dependencies for Tailwind CSS CLI
│   ├── Trunk.toml            # Trunk bundler build & watch configuration
│   └── Cargo.toml
├── .gitignore                # Production-grade git ignore rules
├── Cargo.toml                # Root workspace configuration
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

Ensure the following tools are installed on your machine:
- **Rust toolchain:** `rustup toolchain install stable`
- **WASM compilation target:** `rustup target add wasm32-unknown-unknown`
- **Trunk:** `cargo install trunk`
- **Node.js & npm:** [Node.js (v18+)](https://nodejs.org/)

---

### Installation & Local Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/vault-note.git
   cd vault-note
   ```

2. **Install frontend CSS dependencies:**
   ```bash
   cd frontend
   npm install
   cd ..
   ```

3. **Start the Backend API Server:**
   ```bash
   cargo run -p backend
   ```
   *The Axum backend will start listening on `http://127.0.0.1:3000`.*

4. **Start the Frontend Web App (in a separate terminal):**
   ```bash
   cd frontend
   trunk serve
   ```
   *The Trunk dev server will build the WASM binary, compile Tailwind styles, and serve the application on `http://127.0.0.1:8080`.*

5. Open [http://127.0.0.1:8080](http://127.0.0.1:8080) in your browser.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| `Ctrl + S` | Save current note to backend | Global |

---

## 🔒 Planned Firebase Authentication Architecture

When ready to integrate authentication:
1. **Frontend Authentication Flow:**
   - A dedicated auth component using Firebase JS SDK / REST endpoints will allow users to sign in using their Google account or verify their phone number via SMS OTP.
   - Upon successful sign-in, Firebase issues an ID token (`JWT`).
2. **Backend Protection:**
   - All protected requests (`POST /api/notes`, `GET /api/notes`) will include the `Authorization: Bearer <ID_TOKEN>` header.
   - The Axum backend will validate the token signature and associate notes with the user's unique Firebase UID.

---

## 📄 License

This project is licensed under the MIT License.
