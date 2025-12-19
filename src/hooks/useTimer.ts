import { useState, useEffect, useRef, useCallback } from "react";

export function useTimer(initialDurationMs: number | null = null) {
  const [elapsedMs, setElapsedMs] = useState(initialDurationMs ?? 0);
  const [isRunning, setIsRunning] = useState(false);
  const intervalRef = useRef<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const offsetRef = useRef(0);

  const start = useCallback((backendDurationMs?: number) => {
    // If we have a backend duration, use it as our starting offset
    if (backendDurationMs !== undefined) {
      offsetRef.current = backendDurationMs;
    }
    startTimeRef.current = performance.now();
    setIsRunning(true);
  }, []);

  const stop = useCallback(() => {
    setIsRunning(false);
    if (intervalRef.current) {
      cancelAnimationFrame(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const reset = useCallback(() => {
    stop();
    setElapsedMs(0);
    offsetRef.current = 0;
  }, [stop]);

  const syncWithBackend = useCallback((backendDurationMs: number) => {
    offsetRef.current = backendDurationMs;
    startTimeRef.current = performance.now();
  }, []);

  useEffect(() => {
    if (!isRunning) return;

    const tick = () => {
      if (startTimeRef.current !== null) {
        const now = performance.now();
        const elapsed = offsetRef.current + (now - startTimeRef.current);
        setElapsedMs(elapsed);
      }
      intervalRef.current = requestAnimationFrame(tick);
    };

    intervalRef.current = requestAnimationFrame(tick);

    return () => {
      if (intervalRef.current) {
        cancelAnimationFrame(intervalRef.current);
      }
    };
  }, [isRunning]);

  // Format as M:SS
  const formatted = formatTime(elapsedMs);

  return {
    elapsedMs,
    formatted,
    isRunning,
    start,
    stop,
    reset,
    syncWithBackend,
  };
}

function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
