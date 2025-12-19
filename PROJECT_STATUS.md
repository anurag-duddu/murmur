# Murmur - Speech-to-Text App

## Project Overview

**Murmur** is a macOS desktop application built with **Tauri 2** (Rust backend + React 19 frontend) that provides global speech-to-text functionality. Users can press a hotkey anywhere on their system to record audio, transcribe it, optionally enhance it with AI, and auto-paste the result.

### Core Flow
```
Hotkey → Record Audio → Transcribe → (Optional) AI Enhancement → Auto-paste
```

---

## Architecture

### Tech Stack (Updated December 2024)

| Layer | Technology | Version |
|-------|------------|---------|
| **Backend** | Rust (Tauri 2) | 2.x |
| **Frontend Framework** | React | 19.2.3 |
| **UI Components** | shadcn/ui (Radix primitives) | Latest |
| **Styling** | Tailwind CSS | 3.4.17 |
| **Animations** | GSAP | 3.14.2 |
| **Motion Library** | Framer Motion | 12.23.26 |
| **Icons** | Lucide React | 0.561.0 |
| **Build Tool** | Vite | 6.4.1 |
| **Audio** | cpal (cross-platform audio capture) | - |
| **Transcription** | Groq Whisper API (primary), Deepgram (BYOK) | - |
| **AI Enhancement** | Claude API (optional) | - |

### Design System

**Theme**: Warm Dark Theme with sophisticated amber accents
- **Font**: Plus Jakarta Sans (Google Fonts)
- **Background**: Deep blue-charcoal (`225 15% 8%`)
- **Foreground**: Warm cream white (`40 20% 96%`)
- **Primary/Accent**: Warm amber (`32 100% 54%`)
- **Border Radius**: 14px (generous, modern feel)
- **Glass Effects**: Backdrop blur with semi-transparent surfaces

---

## Directory Structure

```
speech-to-text-app/
├── src-tauri/                          # Rust backend
│   ├── src/
│   │   ├── lib.rs                      # Main Tauri commands, app logic, overlay positioning
│   │   ├── main.rs                     # Tauri app entry point
│   │   ├── audio.rs                    # Audio capture, resampling, WAV encoding
│   │   ├── whisper_api.rs              # Groq Whisper integration (PRIMARY)
│   │   ├── deepgram.rs                 # Deepgram API (BYOK fallback)
│   │   ├── whisper_local.rs            # Local Whisper (future - lifetime tier)
│   │   ├── transcription.rs            # Unified transcription router
│   │   ├── config.rs                   # App configuration & preferences
│   │   ├── state.rs                    # App state management
│   │   ├── licensing.rs                # License validation (LemonSqueezy)
│   │   ├── model_manager.rs            # Local model downloads
│   │   ├── permissions.rs              # macOS permissions
│   │   └── claude.rs                   # Claude AI enhancement
│   ├── tauri.conf.json                 # Tauri configuration
│   └── Cargo.toml
├── src/                                # React frontend
│   ├── components/
│   │   ├── ui/                         # shadcn/ui base components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   ├── input.tsx
│   │   │   ├── label.tsx
│   │   │   ├── tabs.tsx
│   │   │   ├── select.tsx
│   │   │   ├── switch.tsx
│   │   │   ├── checkbox.tsx
│   │   │   ├── progress.tsx
│   │   │   ├── radio-group.tsx
│   │   │   └── badge.tsx
│   │   ├── preferences/                # Preferences window components
│   │   │   ├── PreferencesWindow.tsx   # Main preferences container
│   │   │   ├── GeneralTab.tsx          # Recording controls, feedback, language
│   │   │   ├── TranscriptionTab.tsx    # Provider selection (Deepgram/Groq/Local)
│   │   │   ├── AudioTab.tsx            # Microphone selection, level monitor
│   │   │   ├── ApiTab.tsx              # API key management (Deepgram, Groq)
│   │   │   ├── ProviderCard.tsx        # Transcription provider selector card
│   │   │   ├── StickyActionBar.tsx     # Save/Cancel action bar
│   │   │   ├── ModelSection.tsx        # Local model management
│   │   │   ├── LicenseSection.tsx      # License activation
│   │   │   └── index.ts
│   │   ├── overlay/                    # Recording overlay components
│   │   │   ├── OverlayWindow.tsx       # Overlay container with state
│   │   │   ├── OverlayPill.tsx         # Recording pill UI
│   │   │   ├── RecordingDot.tsx        # Animated recording indicator
│   │   │   ├── Waveform.tsx            # Audio level visualization
│   │   │   ├── Timer.tsx               # Recording duration timer
│   │   │   └── index.ts
│   │   ├── onboarding/                 # Onboarding flow components
│   │   │   ├── OnboardingWindow.tsx    # Onboarding container
│   │   │   ├── StepIndicator.tsx       # Progress steps
│   │   │   ├── MicrophoneSelector.tsx  # Microphone permission & selection
│   │   │   ├── PermissionCard.tsx      # Permission status cards
│   │   │   ├── SuccessScreen.tsx       # Completion screen
│   │   │   └── index.ts
│   │   └── shared/                     # Shared components
│   │       ├── LanguageChips.tsx       # Language display chips
│   │       └── LanguageGrid.tsx        # Language selection grid
│   ├── hooks/                          # React hooks
│   │   ├── index.ts                    # Hook exports
│   │   ├── usePreferences.ts           # Preferences state management
│   │   ├── useRecordingState.ts        # Recording state hook
│   │   ├── useAudioLevel.ts            # Audio level subscription
│   │   ├── useTimer.ts                 # Timer hook
│   │   ├── usePermissions.ts           # Permission status hook
│   │   ├── useModelStatus.ts           # Model download status
│   │   ├── useLicenseInfo.ts           # License info hook
│   │   └── useGsapAnimations.ts        # GSAP animation hooks
│   ├── types/                          # TypeScript types
│   │   ├── index.ts                    # Type exports
│   │   ├── preferences.ts              # Preferences, languages, hotkeys
│   │   ├── recording.ts                # Recording state types
│   │   ├── model.ts                    # Model status types
│   │   ├── license.ts                  # License info types
│   │   └── permissions.ts              # Permission status types
│   ├── lib/                            # Utilities
│   │   ├── tauri.ts                    # Tauri command/event wrappers
│   │   └── utils.ts                    # cn() helper for classnames
│   ├── styles/
│   │   └── globals.css                 # Global styles, theme, animations
│   ├── index.tsx                       # Main app entry (preferences)
│   ├── overlay.tsx                     # Overlay entry point
│   └── onboarding.tsx                  # Onboarding entry point
├── index.html                          # Main window HTML
├── overlay.html                        # Overlay window HTML
├── onboarding.html                     # Onboarding window HTML
├── package.json
├── tailwind.config.js
├── tsconfig.json
├── vite.config.ts
└── PROJECT_STATUS.md                   # This file
```

---

## Component Architecture

### Windows (Multi-Window Tauri App)

| Window | Entry Point | Purpose |
|--------|-------------|---------|
| **Main (Preferences)** | `index.html` → `index.tsx` | Settings and configuration UI |
| **Overlay** | `overlay.html` → `overlay.tsx` | Recording indicator pill |
| **Onboarding** | `onboarding.html` → `onboarding.tsx` | First-time setup wizard |

### Preferences Window Tabs

```
┌─────────────────────────────────────────────────┐
│  Murmur                           ⌘S to save    │
├─────────────────────────────────────────────────┤
│  [General] [Transcription] [Audio] [API Keys]   │
├─────────────────────────────────────────────────┤
│                                                 │
│  Tab Content Area                               │
│  - GeneralTab: Recording mode, hotkey,          │
│    feedback toggles, language settings          │
│  - TranscriptionTab: Provider cards             │
│  - AudioTab: Mic selection, level monitor       │
│  - ApiTab: Deepgram & Groq API keys             │
│                                                 │
├─────────────────────────────────────────────────┤
│  [Cancel]                      [Save Changes]   │
└─────────────────────────────────────────────────┘
```

### Overlay Window

```
┌──────────────────────────────────────────────────┐
│  ⚫ ▁▂▃▅▃▂▁  00:05  Recording...           [×]  │
└──────────────────────────────────────────────────┘
      │    │      │        │                  │
      │    │      │        │                  └─ Cancel button
      │    │      │        └─ Status message
      │    │      └─ Timer (mm:ss)
      │    └─ Waveform (GSAP animated)
      └─ Recording dot (GSAP pulsing)
```

**Positioning**: Center-bottom of screen, 300px offset from bottom
**Background**: Dark glass effect (`rgba(30,30,30,0.95)` with backdrop blur)

---

## GSAP Animation System

### Animation Hooks (`src/hooks/useGsapAnimations.ts`)

| Hook | Purpose | Components |
|------|---------|------------|
| `useOverlayAnimation()` | Overlay entrance/exit (bounce-in, fade-out) | OverlayWindow |
| `useRecordingPulse(isRecording)` | Pulsing dot with glow effect | RecordingDot |
| `useWaveformAnimation(level, count, ref)` | Smooth bar height transitions | Waveform |
| `useEntranceAnimation(options)` | Generic slide-in animation | PreferencesWindow header |
| `useTabTransition(activeTab)` | Tab content fade+slide | PreferencesWindow |
| `useCardSelectAnimation()` | Selection pulse + checkmark pop-in | ProviderCard |
| `useStaggerReveal()` | Staggered list reveal animation | Lists |

### Animation Presets

```typescript
ANIMATION_PRESETS = {
  overlayEnter: { duration: 0.5, ease: "back.out(1.7)", scale: 0.8→1, y: 20→0 },
  overlayExit: { duration: 0.3, ease: "power2.in", scale: 1→0.95, y: 0→10 },
  tabEnter: { duration: 0.4, ease: "power3.out", y: 12→0 },
  cardSelect: { duration: 0.3, scale: [1, 1.02, 1] },
  staggerReveal: { duration: 0.4, stagger: 0.08, y: 15→0 },
}
```

### Components with GSAP Animations

1. **OverlayWindow** (`src/components/overlay/OverlayWindow.tsx`)
   - Bounce-in entrance animation when overlay appears
   - Uses `useOverlayAnimation()` hook

2. **RecordingDot** (`src/components/overlay/RecordingDot.tsx`)
   - Organic pulse with glowing box-shadow when recording
   - Uses `useRecordingPulse()` hook

3. **Waveform** (`src/components/overlay/Waveform.tsx`)
   - GSAP-powered smooth bar height animations
   - Uses `useWaveformAnimation()` hook

4. **PreferencesWindow** (`src/components/preferences/PreferencesWindow.tsx`)
   - Header entrance animation (slides down)
   - Tab content transitions (fade + slide)
   - Uses `useEntranceAnimation()` hook

5. **ProviderCard** (`src/components/preferences/ProviderCard.tsx`)
   - Selection pulse animation (subtle scale bump)
   - Checkmark icon spin-in with `back.out` easing

6. **StickyActionBar** (`src/components/preferences/StickyActionBar.tsx`)
   - Slide-up entrance animation when changes detected
   - Success icon spin-in animation when saved

---

## Type System

### Preferences Interface (`types/preferences.ts`)

```typescript
interface Preferences {
  recording_mode: "push-to-talk" | "toggle";
  hotkey: string;                              // "Option+Space"
  show_indicator: boolean;
  play_sounds: boolean;
  microphone: string;                          // Device ID or "default"
  language: string;                            // "en-US", "mixed", language codes
  deepgram_api_key: string;
  groq_api_key: string;
  anthropic_api_key: string;
  transcription_provider: TranscriptionProvider;
  license_key: string | null;
  onboarding_complete: boolean;
  spoken_languages: string[];                  // ["en", "te", "hi"]
}

type TranscriptionProvider = "deepgram" | "whisperapi" | "whisperlocal";
```

### Recording State (`types/recording.ts`)

```typescript
type RecordingState = "idle" | "recording" | "transcribing" | "enhancing" | "error";

interface StateChangeEvent {
  state: RecordingState;
  message?: string;
  recording_duration_ms?: number;
}
```

### Available Languages

```typescript
// Spoken languages for selection
SPOKEN_LANGUAGES = [
  "English", "Hindi", "Telugu", "Tamil", "Spanish",
  "French", "German", "Japanese", "Chinese", "Korean"
];

// Transcription languages (40+ options)
TRANSCRIPTION_LANGUAGES = [
  "English (US/UK/AU/IN)", "Mixed (your languages)",
  // Indian: Hindi, Telugu, Tamil, Kannada, Malayalam, Bengali, Marathi, Gujarati, Punjabi
  // European: Spanish, French, German, Italian, Portuguese, Dutch, Russian, Polish, etc.
  // Asian: Japanese, Korean, Chinese (Simplified/Traditional), Vietnamese, Thai, etc.
];
```

---

## Tauri API Layer (`src/lib/tauri.ts`)

### Commands

| Command | Purpose |
|---------|---------|
| `getPreferences()` | Load preferences from backend |
| `updatePreferences(prefs)` | Save preferences |
| `startRecording()` | Begin audio capture |
| `stopRecording()` | Stop and transcribe |
| `cancelRecording()` | Abort recording |
| `toggleRecording()` | Toggle recording state |
| `getOverlayState()` | Get current overlay state |
| `checkPermissions()` | Check mic/accessibility permissions |
| `getMicrophones()` | List available microphones |
| `testDeepgramKey(apiKey)` | Validate Deepgram API key |
| `getModelStatus()` | Get local model download status |
| `downloadModel()` | Start model download |
| `validateLicense(key)` | Validate license key |
| `showPreferences()` | Open preferences window |

### Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `state-changed` | `StateChangeEvent` | Recording state updates |
| `audio-level` | `{ level: number }` | Real-time audio levels (0-1) |
| `model-download-progress` | `ModelDownloadProgress` | Model download progress |
| `transcription-complete` | `TranscriptionCompleteEvent` | Transcription finished |
| `recording-error` | `RecordingErrorEvent` | Error occurred |
| `toggle-recording` | - | Menu bar toggle event |
| `shortcut-start/stop/toggle` | - | Global shortcut events |

---

## Business Model (3 Tiers)

| Tier | Transcription Method | App Size | Requirements |
|------|---------------------|----------|--------------|
| **BYOK (Free)** | Deepgram API | ~15MB | User's Deepgram API key |
| **Subscription** | Groq Whisper API | ~15MB | GROQ_API_KEY |
| **Lifetime** | Local Whisper model | ~15MB + 550MB model | Downloads ggml-large-v3-turbo |

**Current Implementation**:
- ✅ BYOK (Deepgram) - Fully working
- ✅ Subscription (Groq Whisper) - Fully working
- 🔲 Local Whisper - Scaffolded, not connected

---

## Transcription Modes

### Two Language Modes

1. **Native Mode** (Select specific language)
   - Strict transcription in the selected language
   - Output in **native script** (e.g., Hindi → हिंदी, Telugu → తెలుగు)
   - Uses language-specific prompts to ensure correct script
   - Example: Select "Telugu" → "నా పేరు అనురాగ్"

2. **Mixed Mode** (Select "Mixed - your languages")
   - Auto-detects among user's **spoken languages** (from settings)
   - Output is **romanized** (Hinglish, Tenglish, etc.)
   - Uses `verbose_json` to get detected language and confidence
   - Example: English + Telugu → "Na peru Anurag" (romanized)

### Key File: `src-tauri/src/whisper_api.rs`
```rust
// Native mode: Sets language parameter, uses native script prompts
transcribe_native_mode(audio_wav, api_key, lang_code)

// Mixed mode: No language param, auto-detect, romanizes output
transcribe_mixed_mode(audio_wav, api_key, spoken_languages)
```

### Spoken Languages
- Stored in `preferences.spoken_languages` as `["en", "te", "hi"]`
- User sets these during onboarding or in Settings → "Languages I Speak"
- **Critical**: Mixed mode only works well if user adds their languages

---

## UI/UX Features

### Preferences Window
- **Responsive design** - Works on various window sizes with `px-4 sm:px-6` padding
- **Tab-based navigation** - General, Transcription, Audio, API Keys
- **Provider cards** - Visual selection with gradient icons (DG=emerald, GQ=orange)
- **API key management** - Show/hide toggle, test connection, validation feedback
- **Audio level monitor** - Real-time 24-segment meter with color zones
- **Language selection** - Chips display, expandable grid editor
- **Sticky action bar** - Save/Cancel with unsaved changes indicator

### Overlay
- **Floating pill design** - Dark glass effect (`bg-[rgba(30,30,30,0.95)]`)
- **Center-bottom positioning** - 300px offset from screen bottom
- **GSAP entrance animation** - Bounces in with `back.out(1.7)` easing
- **Recording indicator** - Pulsing red dot with glow box-shadow
- **Audio waveform** - 12-bar visualization with smooth GSAP transitions
- **Timer** - Real-time duration display (tabular-nums font)
- **Quick actions** - Click to stop, cancel button

### Design Tokens (`src/styles/globals.css`)

```css
:root {
  /* Warm Dark Theme - Sophisticated & Modern */
  --background: 225 15% 8%;           /* Deep blue-charcoal */
  --foreground: 40 20% 96%;           /* Warm cream white */
  --card: 225 14% 11%;
  --primary: 32 100% 54%;             /* Warm amber */
  --accent: 32 100% 54%;
  --muted: 225 12% 14%;
  --muted-foreground: 225 8% 52%;
  --border: 225 12% 17%;
  --destructive: 0 72% 55%;
  --success: 152 69% 45%;
  --warning: 38 95% 55%;
  --radius: 14px;
}
```

### CSS Utility Classes
- `.glass` - Glass effect with backdrop blur
- `.glass-dark` - Dark glass for overlay
- `.gradient-text` - Amber gradient text
- `.glow-accent` / `.glow-sm` - Glow effects
- `.tabular-nums` - Monospace numbers for timers
- `.scrollbar-hide` - Hidden scrollbars
- `.animate-fade-in` / `.animate-slide-up` / `.animate-scale-in` - CSS animations

---

## Current Status (as of December 2024)

### Working ✅
- [x] Push-to-talk and toggle recording modes
- [x] Global hotkey (Option+Space default)
- [x] Groq Whisper API transcription
- [x] Deepgram BYOK transcription
- [x] Native mode with language-specific prompts
- [x] Mixed mode with romanization
- [x] Auto-paste via keystroke injection
- [x] React 19 + shadcn/ui component system
- [x] GSAP animation system (7 custom hooks)
- [x] Warm dark theme with responsive design
- [x] Multi-window Tauri app (Preferences, Overlay, Onboarding)
- [x] API key testing (Deepgram)
- [x] Audio level visualization
- [x] Onboarding flow
- [x] Settings persistence
- [x] Dynamic overlay positioning (center-bottom, 300px offset)

### Partially Working 🔲
- [ ] Claude AI enhancement (requires active API credits)
- [ ] Local Whisper model (scaffolded, not connected)
- [ ] License validation (scaffolded)
- [ ] Groq API key testing (not yet implemented)

### Known Issues
1. **Mixed mode translation**: Whisper sometimes translates instead of transcribes for code-switched speech
2. **Claude API credits**: Enhancement fails gracefully if no credits
3. **Accessibility permission**: Must re-add after rebuilding app (code signature changes)
4. **Old "auto" language value**: Treated as "mixed" mode for backward compatibility

---

## Configuration

### Environment Variables (`.env`)
```bash
GROQ_API_KEY=gsk_...           # Primary - Whisper transcription
ANTHROPIC_API_KEY=sk-ant-...   # Optional - Claude enhancement
DEEPGRAM_API_KEY=...           # BYOK fallback
```

### Preferences File Location
```
~/Library/Application Support/murmur/preferences.json
```

### Key Preferences
```json
{
  "recording_mode": "toggle",
  "hotkey": "Option+Space",
  "show_indicator": true,
  "play_sounds": true,
  "microphone": "default",
  "language": "en-US",
  "spoken_languages": ["en", "te"],
  "transcription_provider": "whisperapi",
  "deepgram_api_key": "",
  "groq_api_key": "",
  "onboarding_complete": true
}
```

---

## Development Commands

```bash
# Start dev server (auto-rebuilds on changes)
cd speech-to-text-app
npm run tauri dev

# Build for production
npm run tauri build

# Type check and build frontend
npm run build  # Runs tsc && vite build

# Check Rust compilation
cd src-tauri && cargo check

# Kill stuck dev server
lsof -ti:1420 | xargs kill -9
```

---

## Key Technical Decisions

### 1. React 19 + shadcn/ui
- **Why**: Modern React with concurrent features, Radix-based accessible components
- **Benefit**: Type-safe, customizable, consistent design system

### 2. GSAP for Animations
- **Why**: Professional-grade animations, better performance than CSS
- **Where**: Overlay entrance, recording dot pulse, waveform, tab transitions, card selection
- **Hook-based**: Reusable animation hooks for consistency

### 3. Groq over OpenAI/Replicate
- **Why**: Groq is 10-50x faster than other Whisper APIs
- **Model**: `whisper-large-v3-turbo` (best speed/accuracy balance)

### 4. Multi-Window Architecture
- **Why**: Separate concerns - preferences window vs. lightweight overlay
- **Benefit**: Overlay can be always-on-top without blocking preferences

### 5. Dynamic Overlay Positioning
- **Why**: Fixed coordinates don't work across different screen sizes
- **How**: `position_overlay_center_bottom()` in Rust calculates from primary monitor

### 6. snake_case Preferences
- **Why**: Rust backend uses snake_case, must match exactly
- **Note**: Frontend types use snake_case to match (not camelCase)

---

## File Reference

### Backend (Rust)

| File | Purpose |
|------|---------|
| `lib.rs` | Main entry, Tauri commands, recording logic, overlay positioning |
| `main.rs` | Tauri app entry point |
| `whisper_api.rs` | Groq Whisper client with native/mixed modes |
| `audio.rs` | Audio capture, resampling (48kHz→16kHz), WAV encoding |
| `config.rs` | StoredPreferences, AppConfig, TranscriptionProvider enum |
| `deepgram.rs` | Deepgram API client (BYOK) |
| `transcription.rs` | Unified transcription router |
| `whisper_local.rs` | Local Whisper scaffolding (future) |
| `licensing.rs` | LemonSqueezy license validation |
| `state.rs` | RecordingState enum, ErrorEvent types |
| `permissions.rs` | macOS microphone/accessibility permission checks |
| `model_manager.rs` | Local model download logic |
| `claude.rs` | Claude AI enhancement integration |

### Frontend (React/TypeScript)

| File | Purpose |
|------|---------|
| `lib/tauri.ts` | Tauri command/event wrappers with TypeScript types |
| `lib/utils.ts` | `cn()` classname utility (clsx + tailwind-merge) |
| `hooks/useGsapAnimations.ts` | 7 GSAP animation hooks |
| `hooks/usePreferences.ts` | Preferences state management with save/reset |
| `types/preferences.ts` | Preferences interface, language constants, hotkey options |
| `styles/globals.css` | Theme, CSS variables, utility classes, keyframe animations |

---

## Prompt Engineering (whisper_api.rs)

### Native Mode Prompts
Language-specific prompts to ensure native script output:
```rust
"hi" => "हिंदी में ट्रांसक्राइब करें। मेरा नाम अनुराग है।"
"te" => "తెలుగులో ట్రాన్స్క్రిప్ట్ చేయండి. నా పేరు అనురాగ్."
```

### Mixed Mode Prompt
```
"This speaker uses these languages: {lang_names}. TRANSCRIBE (do NOT translate).
Output the exact words spoken in the original language. Never convert one language to another."
```

---

## Common Issues & Solutions

### Issue: Port 1420 already in use
```bash
lsof -ti:1420 | xargs kill -9
```

### Issue: Text not being inserted
**Solution**: Re-add app to System Settings → Privacy & Security → Accessibility

### Issue: API key not working in production
**Cause**: Production builds don't read `.env`
**Solution**: Store in preferences or keychain

### Issue: Overlay cut off at edges
**Solution**: Dynamic positioning with `position_overlay_center_bottom()` function

### Issue: "unsupported language: auto"
**Solution**: Code treats "auto" as "mixed" mode (backward compat)

### Issue: Telugu speech outputs English translation
**Solution**: User must add Telugu to "Languages I Speak" in Settings

---

## Testing Checklist

Before releasing:
- [ ] English transcription (native mode)
- [ ] Hindi transcription (native mode, देवनागरी output)
- [ ] Telugu transcription (native mode, తెలుగు output)
- [ ] Mixed mode with multiple spoken languages
- [ ] Push-to-talk mode
- [ ] Toggle mode
- [ ] Auto-paste into different apps
- [ ] Settings persistence
- [ ] Overlay GSAP animations (entrance, pulse, waveform)
- [ ] Tab transitions in preferences
- [ ] Provider card selection animation
- [ ] API key testing (Deepgram)
- [ ] Onboarding flow completion

---

## Future Work

### High Priority
- [ ] Connect local Whisper model for lifetime tier
- [ ] Complete license validation flow
- [ ] Add Groq API key testing
- [ ] Model download UI with progress bar

### Medium Priority
- [ ] Improve mixed mode accuracy
- [ ] Usage analytics
- [ ] More animation polish
- [ ] Keyboard shortcuts in preferences

### Low Priority
- [ ] Windows/Linux support
- [ ] Custom hotkey recording
- [ ] Audio preprocessing for noise reduction

---

## Notes for New Sessions

When starting a new Claude session:

1. **Reference this file**: "Read PROJECT_STATUS.md for context"
2. **Key constraint**: Groq Whisper is primary transcription, Deepgram is BYOK
3. **Tech stack**: Tauri 2, React 19, shadcn/ui, GSAP, Tailwind
4. **Design**: Warm dark theme, Plus Jakarta Sans font, amber accents
5. **Type safety**: Preferences use snake_case to match Rust backend
6. **Current focus**: Polish, animations, local model integration

### Quick Context
> Murmur is a macOS speech-to-text app using Groq's Whisper API. Built with
> Tauri 2 + React 19 + shadcn/ui. Features GSAP animations (7 custom hooks),
> warm dark theme, multi-window architecture. Two transcription modes: Native
> (single language, native script) and Mixed (auto-detect, romanized output).
