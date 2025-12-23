import { cn } from "@/lib/utils";
import { RecordingDot } from "./RecordingDot";
import { Waveform } from "./Waveform";
import { Timer } from "./Timer";
import { X } from "lucide-react";
import type { RecordingState, DictationMode } from "@/types";

interface OverlayPillProps {
  state: RecordingState;
  mode: DictationMode;
  statusMessage: string;
  audioLevel: number;
  elapsedSeconds: number;
  onStop: () => void;
  onCancel: () => void;
}

export function OverlayPill({
  state,
  mode,
  statusMessage,
  audioLevel,
  elapsedSeconds,
  onStop,
  onCancel,
}: OverlayPillProps) {
  const canStop = state === "recording";
  const isCommandMode = mode === "command";

  // Determine dot state: use "command" (blue) for Command Mode recording
  const dotState =
    state === "recording" ? (isCommandMode ? "command" : "recording") :
    state === "transcribing" || state === "enhancing" || state === "transforming" ? "processing" :
    state === "idle" && statusMessage === "Done!" ? "done" :
    state === "error" ? "error" : "idle";

  const handlePillClick = () => {
    if (canStop) {
      onStop();
    }
  };

  return (
    <div
      onClick={handlePillClick}
      className={cn(
        "flex items-center gap-3 rounded-full px-5 py-3",
        "bg-[rgba(30,30,30,0.95)] backdrop-blur-xl",
        "border border-white/10 shadow-elevated",
        "transition-colors duration-150",
        canStop && "cursor-pointer hover:bg-[rgba(40,40,40,0.95)]",
        !canStop && "cursor-default"
      )}
      title={canStop ? "Click to stop and transcribe" : undefined}
    >
      <RecordingDot state={dotState} />

      <Waveform audioLevel={audioLevel} barCount={12} />

      <Timer seconds={elapsedSeconds} />

      <span className="whitespace-nowrap text-[13px] text-white/70">
        {statusMessage}
      </span>

      <button
        onClick={(e) => {
          e.stopPropagation();
          onCancel();
        }}
        className={cn(
          "flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full",
          "bg-white/10 text-white/60",
          "transition-all duration-150",
          "hover:bg-white/20 hover:text-white"
        )}
        title="Cancel"
      >
        <X className="h-3 w-3" />
      </button>
    </div>
  );
}
