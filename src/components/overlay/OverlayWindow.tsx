import { useState, useEffect, useCallback, useRef } from "react";
import { OverlayPill } from "./OverlayPill";
import { tauriCommands, tauriEvents } from "@/lib/tauri";
import { useOverlayAnimation } from "@/hooks";
import type { RecordingState, DictationMode } from "@/types";

interface OverlayState {
  state: RecordingState;
  message: string;
  recordingDurationMs: number;
  mode: DictationMode;
}

export function OverlayWindow() {
  const [overlayState, setOverlayState] = useState<OverlayState>({
    state: "idle",
    message: "",
    recordingDurationMs: 0,
    mode: "dictation",
  });
  const [audioLevel, setAudioLevel] = useState(0);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);

  // GSAP entrance animation
  const { containerRef } = useOverlayAnimation();

  // Timer refs for accurate tracking
  const backendOffsetRef = useRef(0);
  const syncTimeRef = useRef(Date.now());
  const timerIntervalRef = useRef<number | null>(null);

  // Start timer with backend offset
  const startTimer = useCallback((backendDurationMs: number) => {
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current);
    }
    backendOffsetRef.current = backendDurationMs;
    syncTimeRef.current = Date.now();

    const updateTimer = () => {
      const elapsedMs = backendOffsetRef.current + (Date.now() - syncTimeRef.current);
      setElapsedSeconds(Math.floor(elapsedMs / 1000));
    };

    timerIntervalRef.current = window.setInterval(updateTimer, 100);
    updateTimer();
  }, []);

  // Stop timer
  const stopTimer = useCallback(() => {
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current);
      timerIntervalRef.current = null;
    }
  }, []);

  // Apply state changes
  const applyState = useCallback((
    state: RecordingState,
    message: string | undefined,
    recordingDurationMs: number,
    mode: DictationMode = "dictation"
  ) => {
    setOverlayState({ state, message: message || "", recordingDurationMs, mode });

    if (state === "recording") {
      startTimer(recordingDurationMs || 0);
    } else {
      stopTimer();
    }
  }, [startTimer, stopTimer]);

  // Initialize: get state from backend
  useEffect(() => {
    const init = async () => {
      try {
        const initialState = await tauriCommands.getOverlayState();
        applyState(
          initialState.state,
          initialState.message,
          initialState.recording_duration_ms || 0,
          initialState.mode || "dictation"
        );
      } catch (e) {
        console.error("Failed to get initial overlay state:", e);
      }
    };
    init();

    return () => stopTimer();
  }, [applyState, stopTimer]);

  // Listen for state changes
  useEffect(() => {
    let cleanup: (() => void) | undefined;

    tauriEvents.onStateChanged((event) => {
      applyState(
        event.state,
        event.message,
        event.recording_duration_ms || 0,
        event.mode || "dictation"
      );
    }).then((unsub) => {
      cleanup = unsub;
    });

    return () => cleanup?.();
  }, [applyState]);

  // Listen for audio levels
  useEffect(() => {
    let cleanup: (() => void) | undefined;

    tauriEvents.onAudioLevel((level) => {
      setAudioLevel(level);
    }).then((unsub) => {
      cleanup = unsub;
    });

    return () => cleanup?.();
  }, []);

  // Handle stop recording
  const handleStop = useCallback(async () => {
    try {
      await tauriCommands.stopRecording();
    } catch (e) {
      console.error("Stop error:", e);
    }
  }, []);

  // Handle cancel recording
  const handleCancel = useCallback(async () => {
    try {
      await tauriCommands.cancelRecording();
    } catch (e) {
      console.error("Cancel error:", e);
    }
  }, []);

  // Status message based on state and mode
  const isCommandMode = overlayState.mode === "command";
  const statusMessage =
    overlayState.state === "recording" ? (isCommandMode ? "Command Mode" : "Recording...") :
    overlayState.state === "transcribing" ? (overlayState.message || "Transcribing...") :
    overlayState.state === "enhancing" ? (overlayState.message || "Enhancing...") :
    overlayState.state === "transforming" ? (overlayState.message || "Transforming...") :
    overlayState.state === "idle" && overlayState.message === "Done!" ? "Done!" :
    overlayState.state === "error" ? "Error" :
    overlayState.message || "";

  return (
    <div className="flex h-full w-full items-center justify-center">
      <div ref={containerRef} className="w-fit">
        <OverlayPill
          state={overlayState.state}
          mode={overlayState.mode}
          statusMessage={statusMessage}
          audioLevel={audioLevel}
          elapsedSeconds={elapsedSeconds}
          onStop={handleStop}
          onCancel={handleCancel}
        />
      </div>
    </div>
  );
}
