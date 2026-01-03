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
import type { AuthState, UserInfo } from "@/types/auth";

// ============================================================================
// TAURI COMMANDS
// ============================================================================

export const tauriCommands = {
  // Preferences
  getPreferences: () => invoke<Preferences>("get_preferences"),
  updatePreferences: (preferences: Preferences) =>
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
  requestMicrophonePermission: () => invoke<boolean>("request_microphone_permission"),
  requestAccessibilityPermission: () => invoke<boolean>("request_accessibility_permission"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
  getMicrophones: () => invoke<MicrophoneDevice[]>("get_microphones"),
  setSelectedMicrophone: (deviceId: string) =>
    invoke<void>("set_selected_microphone", { device_id: deviceId }),

  // Onboarding
  isOnboardingComplete: () => invoke<boolean>("is_onboarding_complete"),
  needsReauthorization: () => invoke<boolean>("needs_reauthorization"),
  completeOnboarding: () => invoke<void>("complete_onboarding"),
  restartApp: () => invoke<void>("restart_app"),

  // Window
  showPreferences: () => invoke<void>("show_preferences"),

  // Authentication
  getAuthState: () => invoke<AuthState>("get_auth_state"),
  startAuth: () => invoke<void>("start_auth"),
  logout: () => invoke<void>("logout"),
  getUserInfo: () => invoke<UserInfo | null>("get_user_info"),
  isAuthenticated: () => invoke<boolean>("is_authenticated"),
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

  // Authentication state changes
  onAuthStateChanged: (callback: (state: AuthState) => void): Promise<UnlistenFn> =>
    listen<AuthState>("auth-state-changed", (e) => callback(e.payload)),
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
