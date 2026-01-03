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
    <div
      className={cn(
        "relative flex items-center gap-[3px] h-8 px-1",
        className
      )}
    >
      {/* Subtle glow behind active bars */}
      <div
        className="absolute inset-0 rounded-lg opacity-30 blur-md"
        style={{
          background: `radial-gradient(ellipse at center, hsl(var(--primary) / ${Math.min(audioLevel * 0.4, 0.3)}) 0%, transparent 70%)`,
        }}
      />
      {Array.from({ length: barCount }).map((_, index) => (
        <span
          key={index}
          ref={(el) => { barsRef.current[index] = el; }}
          className="relative w-1 rounded-sm bg-gradient-to-t from-primary via-primary/90 to-amber-300"
          style={{ height: `${BASE_HEIGHT}px`, minHeight: `${BASE_HEIGHT}px` }}
        />
      ))}
    </div>
  );
}
