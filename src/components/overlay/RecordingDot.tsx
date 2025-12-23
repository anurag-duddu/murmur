import { cn } from "@/lib/utils";
import { useRecordingPulse } from "@/hooks";

type DotState = "recording" | "command" | "processing" | "done" | "idle" | "error";

interface RecordingDotProps {
  state: DotState;
  className?: string;
}

export function RecordingDot({ state, className }: RecordingDotProps) {
  const dotRef = useRecordingPulse(state === "recording" || state === "command");

  return (
    <div
      ref={dotRef}
      className={cn(
        "h-3 w-3 flex-shrink-0 rounded-full",
        state === "recording" && "bg-destructive",
        state === "command" && "bg-blue-500",
        state === "processing" && "bg-warning",
        state === "done" && "bg-success",
        state === "idle" && "bg-muted-foreground",
        state === "error" && "bg-destructive",
        className
      )}
    />
  );
}
