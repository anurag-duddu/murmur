# Murmur - Feature Implementation Status

**Last Updated:** 2025-12-25

---

## Executive Summary

| Phase | Feature | Status | Notes |
|-------|---------|--------|-------|
| 1 | Command Mode | ✅ Complete | Working - select text, speak command, text transforms |
| 2 | Context-Aware Styles | ✅ Complete | Working - detects app, applies appropriate style |
| 3 | IDE Integrations | ⚠️ Incomplete | File tagging not triggering IDE picker |
| - | Performance | ✅ Optimized | Overlay appears in ~15-20ms (was 600ms-1.2s) |

---

## Performance Optimizations ✅ COMPLETE

### Problem Identified
The overlay was taking 600ms-1.2s to appear after pressing the hotkey. This was unacceptable for a voice dictation app.

### Root Cause Analysis
Using timing instrumentation, we identified two bottlenecks:
1. **`get_active_app()`** - Was using `osascript` (~250ms)
2. **`get_selected_text()`** - Uses Accessibility API (500ms-1.2s depending on target app)

### Solutions Implemented

#### 1. Fast App Detection (commit a795794)
- Replaced `osascript` with `lsappinfo` for app detection
- **Before:** ~250ms | **After:** ~10-20ms (12-25x faster)

#### 2. Async Selection Detection (commit 4082bbe)
- Moved selection detection to background thread
- Overlay shows immediately, selection detected while user speaks
- Mode switches from "Recording..." to "Command Mode" when selection found

#### 3. Overlay Positioning Fixes (commit 4082bbe)
- Removed `center: true` from overlay config
- Pre-position overlay at bottom-center during app startup
- CSS starts overlay invisible (opacity: 0, scale: 0.8)
- GSAP animates in smoothly - no flash or wrong position

### Performance Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Hotkey-to-overlay | 600ms - 1.2s | **15-20ms** | **50-80x faster** |
| App detection | 250ms | 10-20ms | 12-25x faster |
| Selection detection | 500ms-1.2s (blocking) | 500ms-1.2s (async) | Non-blocking |

### Key Files Modified
- `src-tauri/src/styles/detection.rs` - lsappinfo instead of osascript
- `src-tauri/src/lib.rs` - Async selection detection architecture
- `src-tauri/tauri.conf.json` - Removed center: true from overlay
- `overlay.html` - CSS for invisible initial state
- `src/components/overlay/OverlayWindow.tsx` - overlay-pill-container class

---

## Phase 1: Command Mode ✅ COMPLETE

### What It Does
When you have text selected and invoke the hotkey, Murmur detects it's in "Command Mode" and uses an LLM to transform the selected text based on your voice command.

### How It Works (Updated Architecture)
1. **Overlay Shows Instantly** - Default to Dictation mode
2. **Selection Detection (Async)** - Happens in background thread while user speaks
3. **Mode Switch** - If selection found, switches to Command Mode (~500ms after overlay)
4. **Intent Classification** - LLM determines if speech is a command or new content
5. **Transformation** - For commands, LLM transforms the selected text
6. **Insertion** - Result replaces the original selection

### Key Files
- `src-tauri/src/groq_llm.rs` - Groq LLM client for transformation/enhancement
- `src-tauri/src/platform/macos/selection.rs` - Accessibility API selection detection
- `src-tauri/src/state.rs` - DictationMode enum, state management
- `src-tauri/src/lib.rs` - Async mode detection in `shortcut_start_recording`
- `src/components/overlay/` - Blue dot for Command Mode, status messages

### Verified Working
- ✅ Select text → hotkey → "make shorter" → text replaced
- ✅ Select text → hotkey → "translate to Spanish" → translated
- ✅ No selection → hotkey → dictation mode activates
- ✅ Blue indicator for Command Mode, red for Dictation
- ✅ Live corrections work ("no wait", "actually", "scratch that")
- ✅ Mode switches dynamically while recording (async detection)

---

## Phase 2: Context-Aware Styles ✅ COMPLETE

### What It Does
Automatically adjusts the dictation style based on which app you're using. Slack gets casual tone, email apps get professional, IDEs get technical, etc.

### How It Works
1. **App Detection** - Captures bundle ID via `lsappinfo` (~10-20ms)
2. **Style Mapping** - Maps bundle IDs to predefined styles (casual, professional, technical, etc.)
3. **Keyword Inference** - Unknown apps matched by keywords in bundle ID (mail→professional, chat→casual)
4. **Enhancement** - Style prompt passed to LLM during text enhancement

### Key Files
- `src-tauri/src/styles/mod.rs` - Style struct, StyleId enum, main API
- `src-tauri/src/styles/detection.rs` - lsappinfo-based app detection (fast)
- `src-tauri/src/styles/builtin.rs` - Built-in styles (casual, professional, technical, etc.)
- `src-tauri/src/styles/mapping.rs` - Bundle ID → Style mappings + keyword inference

### Verified Working
- ✅ VSCode/Cursor → Technical style
- ✅ Slack/Discord → Casual style
- ✅ Mail.app/Outlook → Professional style
- ✅ Unknown apps → Neutral style (no modification)
- ✅ IDE text input services detected correctly (CursorUIViewService, etc.)

---

## Phase 3: IDE Integrations ⚠️ INCOMPLETE

### What Was Built

#### Working Components
1. **IDE Detection** (`src-tauri/src/ide/mod.rs`)
   - Detects 25+ IDEs/terminals by bundle ID
   - Handles text input service bundle IDs (e.g., `CursorUIViewService`)
   - ✅ Working correctly

2. **Programming Dictionary** (`src-tauri/src/ide/dictionary.rs`)
   - 50+ programming terms: "A P I" → "API", "jason" → "JSON"
   - ✅ Working correctly

3. **CLI Syntax Patterns** (`src-tauri/src/ide/cli_syntax.rs`)
   - "dash dash verbose" → "--verbose"
   - "pipe" → "|", "and and" → "&&"
   - ✅ Working correctly

4. **Variable Case Recognition** (`src-tauri/src/ide/variable.rs`)
   - "camel case user name" → "userName"
   - "snake case is active" → "is_active"
   - Supports: camelCase, PascalCase, snake_case, SCREAMING_SNAKE, kebab-case
   - ✅ Working correctly

5. **Workspace File Indexing** (`src-tauri/src/ide/file_index.rs`)
   - Indexes project files respecting .gitignore
   - Limits: 10,000 files max
   - ✅ Index builds correctly (disabled on startup for performance)

6. **File Tagging** (`src-tauri/src/ide/file_tagger.rs`)
   - Converts spoken filenames to @-prefixed references
   - Only tags files that exist in workspace index
   - ⚠️ Tags correctly, but doesn't trigger IDE file picker

### What's NOT Working

**Problem:** File tagging adds `@` prefix correctly, but the IDE (Cursor) doesn't treat it as an interactive file reference.

**Expected:** Saying "open components.json" → outputs `@components.json` → Cursor shows file picker
**Actual:** Saying "open components.json" → outputs `@components.json` → Just plain text, no picker

### Files Created for Phase 3
```
src-tauri/src/ide/
├── mod.rs           # Module interface, IDE detection
├── dictionary.rs    # Programming term lookup (working)
├── cli_syntax.rs    # CLI pattern matching (working)
├── variable.rs      # Case style conversion (working)
├── file_index.rs    # Workspace file indexing (working)
└── file_tagger.rs   # File tagging (tags correctly, doesn't trigger picker)
```

### Dependencies Added
```toml
walkdir = "2"           # Recursive file walking
ignore = "0.4"          # Respects .gitignore
fuzzy-matcher = "0.3"   # Fuzzy string matching
```

---

## Technical Architecture

### State Management
- `AppState` in `src-tauri/src/lib.rs` holds all runtime state
- Uses `Mutex<T>` for thread-safe access
- Key fields: `dictation_mode`, `selected_text`, `active_style`, `active_bundle_id`, `workspace_index`

### Recording Flow (Instant Overlay Architecture)
```
1. HOTKEY PRESSED:
   - Capture active app via lsappinfo (~10-20ms)
   - Show overlay IMMEDIATELY
   - Start audio capture

2. ASYNC (while user speaks):
   - Thread 1: Detect selection via Accessibility API (500ms-1s)
   - Thread 2: Process app context, determine style
   - If selection found → switch to Command Mode

3. AFTER RECORDING:
   - Transcribe audio (Deepgram/Groq Whisper/Local)
   - Apply IDE transformations (if in IDE)
   - Apply style-based enhancement (LLM)
   - Insert result at cursor
```

### LLM Integration
- **Provider:** Groq `llama-3.3-70b-versatile` (free tier, 128K context)
- **Intent Classification:** Determines command vs. dictation
- **Transformation:** Applies voice commands to selected text
- **Enhancement:** Cleans up transcription, applies style

---

## Configuration

### API Keys (environment variables)
- `GROQ_API_KEY` - Required for LLM features
- `DEEPGRAM_API_KEY` - For Deepgram transcription provider

### Hotkey
- Default: Option+Space (configurable in preferences)
- Modes: Toggle or Push-to-Talk

---

## Git Commits

| Commit | Description |
|--------|-------------|
| `4082bbe` | Fix overlay invocation speed and positioning |
| `a795794` | Fix preferences not saving and duplicate tray icon |
| `c92c1e3` | Initial commit: Murmur speech-to-text app |

---

## Next Steps

### For Phase 3 (IDE File Picker)
1. **Research** - How do VS Code/Cursor extensions trigger file picker?
2. **Consider** - Building a companion VS Code extension
3. **Consider** - Using accessibility API to interact with autocomplete
4. **Consider** - Alternative UX (show file list in overlay, user picks)

### Potential Future Improvements
- Further optimize Accessibility API selection detection (if possible)
- Add visual feedback during async mode switch
- Consider caching frequently used apps for even faster detection

---

## Session History

### 2025-12-25 Session
- Diagnosed overlay speed issue using timing instrumentation
- Fixed app detection: osascript → lsappinfo (12-25x faster)
- Fixed selection detection: blocking → async (non-blocking)
- Fixed overlay positioning: no more flash or wrong position
- Hotkey-to-overlay now ~15-20ms instead of 600ms-1.2s
- Pushed all changes to GitHub

### 2025-12-23 Session
- Implemented Phase 3 IDE integrations
- All components work except file picker triggering
- Added workspace auto-detection on startup
- Documented as incomplete pending further research

### Previous Sessions
- Implemented Phase 1 Command Mode (complete)
- Implemented Phase 2 Context-Aware Styles (complete)
- Created PRD documents for all features
