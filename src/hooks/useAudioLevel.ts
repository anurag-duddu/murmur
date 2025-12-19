import { useState, useEffect, useRef } from "react";
import { tauriEvents } from "@/lib/tauri";

interface UseAudioLevelOptions {
  noiseGate?: number;
  attackSmoothing?: number;
  decaySmoothing?: number;
}

export function useAudioLevel(options: UseAudioLevelOptions = {}) {
  const {
    noiseGate = 0.1,
    attackSmoothing = 0.3,
    decaySmoothing = 0.6,
  } = options;

  const [level, setLevel] = useState(0);
  const smoothedRef = useRef(0);

  useEffect(() => {
    const unsubscribe = tauriEvents.onAudioLevel((rawLevel) => {
      // Apply noise gate
      const gatedLevel = rawLevel < noiseGate ? 0 : rawLevel;

      // Apply asymmetric smoothing (fast attack, slow decay)
      const smoothing =
        gatedLevel > smoothedRef.current ? attackSmoothing : decaySmoothing;

      smoothedRef.current =
        smoothedRef.current + (gatedLevel - smoothedRef.current) * (1 - smoothing);

      setLevel(smoothedRef.current);
    });

    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, [noiseGate, attackSmoothing, decaySmoothing]);

  return level;
}
