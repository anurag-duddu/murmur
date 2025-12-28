import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  Preferences,
  StateChangeEvent,
  PermissionStatus,
  MicrophoneDevice,
  TranscriptionCompleteEvent,
  RecordingErrorEvent,
  AudioLevelEvent,
} from "@/types";

// ============================================================================
// TAURI COMMANDS
// ============================================================================

export const tauriCommands = {
  // Preferences
  getPreferences: () => invoke<Preferences>("get_preferences"),
  updatePreferences: (preferences: Preferences) =>
    invoke<void>("update_preferences", { preferences }),
  savePreferences: (preferences: Preferences) =>
    invoke<void>("update_preferences", { preferences }),

  // Recording
  startRecording: () => invoke<void>("start_recording"),
  stopRecording: () => invoke<void>("stop_recording"),
  cancelRecording: () => invoke<void>("cancel_recording"),
  toggleRecording: () => invoke<void>("toggle_recording"),
  getOverlayState: () => invoke<StateChangeEvent>("get_overlay_state"),
  getRecordingState: () => invoke<string>("get_recording_state"),

  // Permissions
  checkPermissions: () => invoke<PermissionStatus>("check_permissions"),
  getPermissionStatus: () => invoke<PermissionStatus>("check_permissions"),
  requestMicrophonePermission: () => invoke<boolean>("request_microphone_permission"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
  getMicrophones: () => invoke<MicrophoneDevice[]>("get_microphones"),
  setSelectedMicrophone: (deviceId: string) =>
    invoke<void>("set_selected_microphone", { device_id: deviceId }),

  // Onboarding
  isOnboardingComplete: () => invoke<boolean>("is_onboarding_complete"),
  completeOnboarding: () => invoke<void>("complete_onboarding"),

  // Window
  showPreferences: () => invoke<void>("show_preferences"),
};

// ============================================================================
// TAURI EVENTS
// ============================================================================

export const tauriEvents = {
  // Recording state changes
  onStateChanged: (callback: (event: StateChangeEvent) => void): Promise<UnlistenFn> =>
    listen<StateChangeEvent>("state-changed", (e) => callback(e.payload)),

  // Audio level for waveform visualization
  onAudioLevel: (callback: (level: number) => void): Promise<UnlistenFn> =>
    listen<AudioLevelEvent>("audio-level", (e) => callback(e.payload.level)),

  // Transcription events
  onTranscriptionComplete: (callback: (data: TranscriptionCompleteEvent) => void): Promise<UnlistenFn> =>
    listen<TranscriptionCompleteEvent>("transcription-complete", (e) => callback(e.payload)),

  // Recording error
  onRecordingError: (callback: (error: RecordingErrorEvent) => void): Promise<UnlistenFn> =>
    listen<RecordingErrorEvent>("recording-error", (e) => callback(e.payload)),

  // Menu bar toggle
  onToggleRecording: (callback: () => void): Promise<UnlistenFn> =>
    listen("toggle-recording", () => callback()),

  // Global shortcut events
  onShortcutStart: (callback: () => void): Promise<UnlistenFn> =>
    listen("shortcut-start", () => callback()),

  onShortcutStop: (callback: () => void): Promise<UnlistenFn> =>
    listen("shortcut-stop", () => callback()),

  onShortcutToggle: (callback: () => void): Promise<UnlistenFn> =>
    listen("shortcut-toggle", () => callback()),
};

// ============================================================================
// WINDOW UTILITIES
// ============================================================================

export const tauriWindow = {
  hide: async () => {
    const window = getCurrentWindow();
    await window.hide();
  },
  show: async () => {
    const window = getCurrentWindow();
    await window.show();
  },
  focus: async () => {
    const window = getCurrentWindow();
    await window.setFocus();
  },
};
