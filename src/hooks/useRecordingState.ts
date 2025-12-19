import { useState, useEffect, useCallback } from "react";
import { tauriCommands, tauriEvents } from "@/lib/tauri";
import type { RecordingState, StateChangeEvent } from "@/types";

export function useRecordingState() {
  const [state, setState] = useState<RecordingState>("idle");
  const [message, setMessage] = useState<string | null>(null);
  const [durationMs, setDurationMs] = useState<number | null>(null);

  useEffect(() => {
    // Get initial state
    tauriCommands.getOverlayState().then((event) => {
      setState(event.state);
      setMessage(event.message ?? null);
      setDurationMs(event.recording_duration_ms ?? null);
    }).catch(console.error);

    // Subscribe to state changes
    const unsubscribe = tauriEvents.onStateChanged((event: StateChangeEvent) => {
      setState(event.state);
      setMessage(event.message ?? null);
      setDurationMs(event.recording_duration_ms ?? null);
    });

    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, []);

  const startRecording = useCallback(async () => {
    try {
      await tauriCommands.startRecording();
    } catch (err) {
      console.error("Failed to start recording:", err);
    }
  }, []);

  const stopRecording = useCallback(async () => {
    try {
      await tauriCommands.stopRecording();
    } catch (err) {
      console.error("Failed to stop recording:", err);
    }
  }, []);

  const cancelRecording = useCallback(async () => {
    try {
      await tauriCommands.cancelRecording();
    } catch (err) {
      console.error("Failed to cancel recording:", err);
    }
  }, []);

  const toggleRecording = useCallback(async () => {
    try {
      await tauriCommands.toggleRecording();
    } catch (err) {
      console.error("Failed to toggle recording:", err);
    }
  }, []);

  return {
    state,
    message,
    durationMs,
    isRecording: state === "recording",
    isProcessing: state === "transcribing" || state === "enhancing",
    isIdle: state === "idle",
    isError: state === "error",
    startRecording,
    stopRecording,
    cancelRecording,
    toggleRecording,
  };
}
