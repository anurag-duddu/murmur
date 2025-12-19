import { cn } from "@/lib/utils";

interface TimerProps {
  seconds: number;
  className?: string;
}

export function Timer({ seconds, className }: TimerProps) {
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  const display = `${minutes}:${secs.toString().padStart(2, "0")}`;

  return (
    <span
      className={cn(
        "min-w-[45px] text-center text-sm font-semibold text-white tabular-nums",
        className
      )}
    >
      {display}
    </span>
  );
}
