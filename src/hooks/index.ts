// Re-export all hooks
export { usePreferences } from "./usePreferences";
export { useRecordingState } from "./useRecordingState";
export { useAudioLevel } from "./useAudioLevel";
export { useTimer } from "./useTimer";
export { usePermissions } from "./usePermissions";
export { useModelStatus } from "./useModelStatus";
export { useLicenseInfo } from "./useLicenseInfo";

// GSAP animation hooks
export {
  useOverlayAnimation,
  useRecordingPulse,
  useTabTransition,
  useCardSelectAnimation,
  useStaggerReveal,
  useWaveformAnimation,
  useEntranceAnimation,
  ANIMATION_PRESETS,
} from "./useGsapAnimations";
