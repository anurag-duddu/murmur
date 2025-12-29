# Murmur - Project Context

## Git Push Authentication

Use the GitHub token from `.env` file to push to the repository:

```bash
git push https://${GITHUB_TOKEN}@github.com/anurag-duddu/murmur.git main
```

The token is stored in `.env` as `GITHUB_TOKEN`. Required scopes: `repo`, `workflow`.

## Build Commands

- Dev: `npm run tauri dev`
- Build: `npm run tauri build`
- Tests: `cd src-tauri && cargo test`

## Architecture

- **Frontend**: React + TypeScript + Vite
- **Backend**: Tauri v2 (Rust)
- **Proxy**: Cloudflare Worker at `murmur-proxy.anurag-ebc.workers.dev`
- **APIs**: Groq (Whisper for STT, LLaMA for LLM)

## Dev vs Prod

- Debug builds (`npm run tauri dev`): Use direct Groq API with `GROQ_API_KEY` from `.env`
- Release builds (`npm run tauri build`): Always use proxy with HMAC auth

## Secrets Location

- Tauri updater keys: `~/.tauri/murmur.key` (private), `~/.tauri/murmur.key.pub` (public)
- GitHub secrets needed: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
