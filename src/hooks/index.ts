// Re-export all hooks
export { usePreferences } from "./usePreferences";
export { useRecordingState } from "./useRecordingState";
export { useAudioLevel } from "./useAudioLevel";
export { useTimer } from "./useTimer";
export { usePermissions } from "./usePermissions";

// GSAP animation hooks
export {
  useOverlayAnimation,
  useRecordingPulse,
  useTabTransition,
  useWaveformAnimation,
  useEntranceAnimation,
  ANIMATION_PRESETS,
} from "./useGsapAnimations";
