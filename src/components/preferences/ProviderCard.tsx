import { useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { Check } from "lucide-react";
import gsap from "gsap";
import type { TranscriptionProvider } from "@/types";

interface ProviderCardProps {
  provider: TranscriptionProvider;
  title: string;
  subtitle: string;
  description: string;
  icon: string;
  iconColor: string;
  tags: { label: string; variant: "success" | "default" | "secondary" }[];
  selected: boolean;
  onSelect: () => void;
}

export function ProviderCard({
  title,
  subtitle,
  description,
  icon,
  iconColor,
  tags,
  selected,
  onSelect,
}: ProviderCardProps) {
  const cardRef = useRef<HTMLButtonElement>(null);
  const checkRef = useRef<HTMLDivElement>(null);
  const prevSelectedRef = useRef(selected);

  // Animate selection change
  useEffect(() => {
    if (prevSelectedRef.current !== selected && cardRef.current) {
      if (selected) {
        // Selection pulse animation
        gsap.to(cardRef.current, {
          keyframes: { scale: [1, 1.02, 1] },
          duration: 0.3,
          ease: "power2.out",
        });
        // Check icon pop-in
        if (checkRef.current) {
          gsap.fromTo(
            checkRef.current,
            { scale: 0, rotate: -180 },
            { scale: 1, rotate: 0, duration: 0.4, ease: "back.out(1.7)" }
          );
        }
      }
      prevSelectedRef.current = selected;
    }
  }, [selected]);

  return (
    <button
      ref={cardRef}
      type="button"
      onClick={onSelect}
      className={cn(
        "group relative w-full overflow-hidden rounded-2xl text-left transition-all duration-200",
        "border-2",
        selected
          ? "border-accent-foreground/30 bg-gradient-to-br from-accent/10 via-accent/5 to-transparent shadow-lg shadow-accent/10"
          : "border-white/[0.06] bg-white/[0.02] hover:border-white/[0.12] hover:bg-white/[0.04]"
      )}
    >
      {/* Subtle gradient overlay on hover */}
      <div className={cn(
        "absolute inset-0 opacity-0 transition-opacity duration-300",
        "bg-gradient-to-br from-white/[0.03] to-transparent",
        "group-hover:opacity-100"
      )} />

      <div className="relative flex items-start gap-4 p-4">
        {/* Icon with gradient */}
        <div
          className={cn(
            "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl",
            "bg-gradient-to-br text-sm font-bold text-white shadow-lg",
            iconColor,
            selected && "ring-2 ring-white/20 ring-offset-2 ring-offset-background"
          )}
        >
          {icon}
        </div>

        {/* Content */}
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-foreground">{title}</h3>
            <span className="rounded-full bg-white/[0.06] px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
              {subtitle}
            </span>
          </div>
          <p className="text-sm leading-relaxed text-muted-foreground/80">
            {description}
          </p>

          {/* Tags */}
          <div className="flex flex-wrap gap-1.5 pt-1">
            {tags.map((tag, i) => (
              <span
                key={i}
                className={cn(
                  "rounded-md px-2 py-0.5 text-[11px] font-medium",
                  tag.variant === "success" && "bg-emerald-500/15 text-emerald-400",
                  tag.variant === "default" && "bg-accent/15 text-accent",
                  tag.variant === "secondary" && "bg-white/[0.06] text-muted-foreground"
                )}
              >
                {tag.label}
              </span>
            ))}
          </div>
        </div>

        {/* Selection indicator */}
        <div
          className={cn(
            "flex h-6 w-6 shrink-0 items-center justify-center rounded-full transition-all duration-200",
            selected
              ? "bg-accent text-white"
              : "border-2 border-white/10"
          )}
        >
          {selected && (
            <div ref={checkRef}>
              <Check className="h-3.5 w-3.5" strokeWidth={3} />
            </div>
          )}
        </div>
      </div>
    </button>
  );
}
