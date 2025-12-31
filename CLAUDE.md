# Murmur - Project Context

## Build Commands

### Development
- `npm run tauri:dev` - Run with hot reload (debug build)
- `npm run tauri:build` - Build release app

### Tests
- `cd src-tauri && cargo test` - Run Rust tests

## Git Workflow

We use standard Git branching:

```
main              Stable, deployable code
  └── feature/*   New features (e.g., feature/voice-commands)
  └── fix/*       Bug fixes (e.g., fix/hotkey-registration)
```

**Workflow:**
1. Create feature branch: `git checkout -b feature/my-feature`
2. Develop and test: `npm run tauri:dev`
3. Build and verify: `npm run tauri:build`
4. Push and create PR: `git push -u origin feature/my-feature`
5. After merge, tag for release: `git tag v0.2.0 && git push --tags`

## Architecture

- **Frontend**: React + TypeScript + Vite
- **Backend**: Tauri v2 (Rust)
- **Proxy**: Cloudflare Worker at `murmur-proxy.anurag-ebc.workers.dev`
- **APIs**: Groq (Whisper for STT, LLaMA for LLM)

## Debug vs Release Builds

- Debug builds (`npm run tauri:dev`): Use direct Groq API with `GROQ_API_KEY` from `.env`
- Release builds (`npm run tauri:build`): Use proxy with HMAC auth

## Git Push Authentication

**Option 1: SSH (recommended)**
```bash
git remote set-url origin git@github.com:anurag-duddu/murmur.git
git push origin main
```

**Option 2: Git credential helper**
```bash
export GITHUB_TOKEN="$(cat .env | grep GITHUB_TOKEN | cut -d'=' -f2)"
git -c credential.helper='!f() { echo "password=${GITHUB_TOKEN}"; }; f' push origin main
```

## Secrets Location

- Tauri updater keys: `~/.tauri/murmur.key` (private), `~/.tauri/murmur.key.pub` (public)
- GitHub secrets needed: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
