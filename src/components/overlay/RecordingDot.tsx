import { cn } from "@/lib/utils";
import { useRecordingPulse } from "@/hooks";

type DotState = "recording" | "command" | "processing" | "done" | "idle" | "error";

interface RecordingDotProps {
  state: DotState;
  className?: string;
}

const stateStyles: Record<DotState, { bg: string; glow: string }> = {
  recording: {
    bg: "bg-destructive",
    glow: "shadow-[0_0_12px_2px_rgba(255,59,48,0.5)]",
  },
  command: {
    bg: "bg-blue-500",
    glow: "shadow-[0_0_12px_2px_rgba(59,130,246,0.5)]",
  },
  processing: {
    bg: "bg-warning",
    glow: "shadow-[0_0_10px_2px_rgba(245,158,11,0.4)]",
  },
  done: {
    bg: "bg-success",
    glow: "shadow-[0_0_10px_2px_rgba(34,197,94,0.4)]",
  },
  idle: {
    bg: "bg-muted-foreground",
    glow: "",
  },
  error: {
    bg: "bg-destructive",
    glow: "shadow-[0_0_12px_2px_rgba(255,59,48,0.5)]",
  },
};

export function RecordingDot({ state, className }: RecordingDotProps) {
  const dotRef = useRecordingPulse(state === "recording" || state === "command");
  const styles = stateStyles[state];

  return (
    <div
      ref={dotRef}
      className={cn(
        "h-3 w-3 flex-shrink-0 rounded-full transition-all duration-200",
        styles.bg,
        styles.glow,
        className
      )}
    />
  );
}
