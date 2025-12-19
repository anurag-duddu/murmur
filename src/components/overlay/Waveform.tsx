import { useRef } from "react";
import { cn } from "@/lib/utils";
import { useWaveformAnimation } from "@/hooks";

interface WaveformProps {
  audioLevel: number;
  barCount?: number;
  className?: string;
}

const BASE_HEIGHT = 4;

export function Waveform({ audioLevel, barCount = 12, className }: WaveformProps) {
  const barsRef = useRef<(HTMLSpanElement | null)[]>([]);

  // Use GSAP for smooth waveform animations
  useWaveformAnimation(audioLevel, barCount, barsRef);

  return (
    <div className={cn("flex items-center gap-[3px] h-8", className)}>
      {Array.from({ length: barCount }).map((_, index) => (
        <span
          key={index}
          ref={(el) => { barsRef.current[index] = el; }}
          className="w-1 rounded-sm bg-gradient-to-t from-primary to-primary/70"
          style={{ height: `${BASE_HEIGHT}px`, minHeight: `${BASE_HEIGHT}px` }}
        />
      ))}
    </div>
  );
}
