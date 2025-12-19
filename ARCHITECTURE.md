# Speech-to-Text App Architecture

## State Machine

```
                    ┌─────────────────────────────────────────────────────────────┐
                    │                                                             │
                    ▼                                                             │
    ┌─────────┐  start   ┌───────────┐  stop   ┌──────────────┐  done   ┌────────┴─────┐
    │  IDLE   │ ───────► │ RECORDING │ ──────► │ TRANSCRIBING │ ──────► │  ENHANCING   │
    └─────────┘          └───────────┘         └──────────────┘         └──────────────┘
         ▲                     │                      │                        │
         │                     │ cancel               │ error                  │ error/done
         │                     ▼                      ▼                        ▼
         └─────────────────────┴──────────────────────┴────────────────────────┘
```

### States

| State | Description | UI | Actions Available |
|-------|-------------|-----|-------------------|
| IDLE | Ready to record | Subtle indicator, greyed out | Start recording |
| RECORDING | Capturing audio | Pulsing waveform, red dot, timer | Stop, Cancel |
| TRANSCRIBING | Sending to Deepgram | "Transcribing..." spinner | Cancel |
| ENHANCING | Sending to Claude | "Enhancing..." spinner | Cancel |
| ERROR | Something failed | Error message | Retry, Dismiss |

### State Transitions

| From | To | Trigger | Side Effects |
|------|-----|---------|--------------|
| IDLE → RECORDING | User starts | Create audio stream, show overlay |
| RECORDING → TRANSCRIBING | User stops | Stop stream, convert WAV, send to Deepgram |
| RECORDING → IDLE | User cancels | Stop stream, discard audio |
| TRANSCRIBING → ENHANCING | Transcript received | Send to Claude |
| TRANSCRIBING → IDLE | Error or cancel | Show error, cleanup |
| ENHANCING → IDLE | Enhanced text received | Copy to clipboard, auto-paste, hide overlay |
| ENHANCING → IDLE | Error | Show raw transcript as fallback |

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           RUST BACKEND                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │   AppState      │    │  AudioRecorder  │    │  API Clients    │     │
│  │                 │    │                 │    │                 │     │
│  │ - state: enum   │───►│ - stream        │    │ - Deepgram      │     │
│  │ - transcript    │    │ - audio_data    │    │ - Claude        │     │
│  │ - error         │    │ - audio_levels  │    │                 │     │
│  └────────┬────────┘    └────────┬────────┘    └─────────────────┘     │
│           │                      │                                      │
│           │    Events            │ Audio Levels (30fps)                 │
│           ▼                      ▼                                      │
│  ┌─────────────────────────────────────────────┐                       │
│  │              Event Emitter                   │                       │
│  │  - state-changed                            │                       │
│  │  - audio-level                              │                       │
│  │  - error                                    │                       │
│  │  - transcription-complete                   │                       │
│  └─────────────────────────────────────────────┘                       │
│                                                                         │
└───────────────────────────────────┬─────────────────────────────────────┘
                                    │
                                    │ Tauri Events
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         TYPESCRIPT FRONTEND                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │  Overlay Window │    │ Preferences Win │    │  Menu Bar       │     │
│  │                 │    │                 │    │                 │     │
│  │ - waveform      │    │ - API keys      │    │ - Start/Stop    │     │
│  │ - timer         │    │ - Hotkey        │    │ - Settings      │     │
│  │ - status text   │    │ - Mode          │    │ - Quit          │     │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Overlay Window Specification

### Window Properties (Tauri)
```json
{
  "label": "overlay",
  "title": "",
  "width": 300,
  "height": 80,
  "visible": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "shadow": false,
  "center": true
}
```

### Overlay UI Design
```
┌────────────────────────────────────────────────────┐
│                                                    │
│   ●  ▁▂▃▅▆▇▅▃▂▁▂▄▆▇▅▃▁   0:05   Processing...    │
│                                                    │
└────────────────────────────────────────────────────┘
     │       │              │          │
     │       │              │          └─ Status text
     │       │              └─ Recording timer
     │       └─ Audio waveform (12 bars)
     └─ Recording indicator dot
```

### Waveform Implementation
- 12 bars with CSS
- Heights driven by audio RMS levels from backend
- Update at 30fps (33ms interval)
- Smooth transitions with CSS

## Data Flow

### Recording Flow
```
1. User clicks "Start Recording"
   └─► Frontend calls invoke("start_recording")
       └─► Backend: state = RECORDING
           └─► Backend: start audio stream
               └─► Backend: emit("state-changed", RECORDING)
                   └─► Frontend: show overlay, start timer

2. Audio capturing (continuous)
   └─► cpal callback receives samples
       └─► Calculate RMS level
           └─► emit("audio-level", level) at 30fps
               └─► Frontend: update waveform bars

3. User clicks "Stop Recording"
   └─► Frontend calls invoke("stop_recording")
       └─► Backend: state = TRANSCRIBING
           └─► Backend: stop audio stream
               └─► Backend: convert to WAV
                   └─► Backend: emit("state-changed", TRANSCRIBING)
                       └─► Frontend: show "Transcribing..."

4. Transcription
   └─► Backend: send to Deepgram
       └─► Backend: receive transcript
           └─► Backend: state = ENHANCING
               └─► Backend: emit("state-changed", ENHANCING)
                   └─► Frontend: show "Enhancing..."

5. Enhancement
   └─► Backend: send to Claude
       └─► Backend: receive enhanced text
           └─► Backend: copy to clipboard
               └─► Backend: state = IDLE
                   └─► Backend: emit("transcription-complete", text)
                       └─► Frontend: hide overlay, auto-paste
```

## Error Handling

### Error Recovery Strategy
| Error | Recovery |
|-------|----------|
| Mic permission denied | Show system preferences, return to IDLE |
| No audio device | List available devices, ask user to select |
| Deepgram API error | Retry once, then show error with audio size info |
| Claude API error | Use raw transcript as fallback |
| Clipboard error | Show text in popup for manual copy |
| Auto-paste fails | Silent fail (clipboard still has text) |

### Error State Management
```rust
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
    pub fallback_text: Option<String>,
}

pub enum ErrorCode {
    MicPermissionDenied,
    NoAudioDevice,
    NoAudioCaptured,
    DeepgramError,
    ClaudeError,
    ClipboardError,
    NetworkError,
}
```

## File Structure

```
src-tauri/
├── src/
│   ├── lib.rs           # Main app, commands, state machine
│   ├── state.rs         # AppState enum, transitions
│   ├── audio.rs         # Audio capture, level metering
│   ├── deepgram.rs      # Deepgram API client
│   ├── claude.rs        # Claude API client
│   ├── config.rs        # Configuration loading
│   └── error.rs         # Error types and handling

src/
├── main.ts              # Main app logic, event listeners
├── overlay.ts           # Overlay window logic
├── overlay.html         # Overlay window HTML
├── index.html           # Preferences window
├── styles.css           # All styles
└── waveform.ts          # Waveform component

```

## Implementation Priority

### Phase 1: Core State Machine (This Session)
1. Create proper state enum in Rust
2. Implement state transitions
3. Add event emission for state changes
4. Fix frontend to listen to backend state

### Phase 2: Floating Overlay (This Session)
1. Add overlay window to tauri.conf.json
2. Create overlay.html with waveform UI
3. Implement show/hide based on state
4. Position at screen center

### Phase 3: Audio Metering (This Session)
1. Add RMS calculation in audio callback
2. Emit audio levels at 30fps
3. Update waveform bars from frontend
4. Add smooth CSS transitions

### Phase 4: Polish (Next Session)
1. Error handling and recovery
2. Hotkey support
3. Device selection
4. Preferences persistence
