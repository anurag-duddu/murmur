export type RecordingState =
  | "idle"
  | "recording"
  | "transcribing"
  | "enhancing"
  | "error";

export interface StateChangeEvent {
  state: RecordingState;
  message?: string;
  recording_duration_ms?: number;
}

export interface TranscriptionCompleteEvent {
  raw_transcript: string;
  enhanced_text: string;
  copied_to_clipboard: boolean;
}

export interface RecordingErrorEvent {
  code: string;
  message: string;
}

export interface AudioLevelEvent {
  level: number; // 0.0 to 1.0
}

// State helpers
export const canStartRecording = (state: RecordingState): boolean => {
  return state === "idle" || state === "error";
};

export const canStopRecording = (state: RecordingState): boolean => {
  return state === "recording";
};

export const isProcessing = (state: RecordingState): boolean => {
  return state === "transcribing" || state === "enhancing";
};
