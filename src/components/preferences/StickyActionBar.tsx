import { useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Check, Loader2, AlertCircle } from "lucide-react";
import gsap from "gsap";

interface StickyActionBarProps {
  hasChanges: boolean;
  isSaving: boolean;
  saveSuccess: boolean;
  onSave: () => void;
  onCancel: () => void;
}

export function StickyActionBar({
  hasChanges,
  isSaving,
  saveSuccess,
  onSave,
  onCancel,
}: StickyActionBarProps) {
  const barRef = useRef<HTMLDivElement>(null);
  const successIconRef = useRef<HTMLDivElement>(null);
  const prevShowRef = useRef(hasChanges || saveSuccess);

  // Animate bar visibility
  useEffect(() => {
    const shouldShow = hasChanges || saveSuccess;

    if (shouldShow !== prevShowRef.current && barRef.current) {
      if (shouldShow) {
        gsap.fromTo(
          barRef.current,
          { y: 20, opacity: 0 },
          { y: 0, opacity: 1, duration: 0.35, ease: "power3.out" }
        );
      } else {
        gsap.to(barRef.current, {
          y: 10, opacity: 0, duration: 0.25, ease: "power2.in"
        });
      }
      prevShowRef.current = shouldShow;
    }
  }, [hasChanges, saveSuccess]);

  // Animate success icon
  useEffect(() => {
    if (saveSuccess && successIconRef.current) {
      gsap.fromTo(
        successIconRef.current,
        { scale: 0, rotate: -90 },
        { scale: 1, rotate: 0, duration: 0.4, ease: "back.out(1.7)" }
      );
    }
  }, [saveSuccess]);

  return (
    <div
      ref={barRef}
      className={cn(
        "sticky bottom-0 left-0 right-0",
        "border-t border-white/[0.06] bg-gradient-to-t from-background via-background to-background/95 backdrop-blur-sm",
        !(hasChanges || saveSuccess) && "pointer-events-none opacity-0"
      )}
    >
      <div className="flex items-center justify-between px-4 sm:px-6 py-3">
        <div className="flex items-center gap-2">
          {saveSuccess ? (
            <span className="flex items-center gap-2 text-sm text-emerald-400">
              <div ref={successIconRef} className="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-500/20">
                <Check className="h-3 w-3" />
              </div>
              Settings saved
            </span>
          ) : hasChanges ? (
            <span className="flex items-center gap-2 text-sm text-muted-foreground">
              <div className="flex h-5 w-5 items-center justify-center rounded-full bg-amber-500/20">
                <AlertCircle className="h-3 w-3 text-amber-400" />
              </div>
              <span className="hidden sm:inline">You have unsaved changes</span>
              <span className="sm:hidden">Unsaved changes</span>
            </span>
          ) : null}
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            onClick={onCancel}
            disabled={isSaving || !hasChanges}
            className="text-muted-foreground hover:text-foreground hover:bg-white/[0.06]"
          >
            Cancel
          </Button>
          <Button
            onClick={onSave}
            disabled={isSaving || !hasChanges}
            className={cn(
              "min-w-[100px] transition-all",
              saveSuccess
                ? "bg-emerald-500 hover:bg-emerald-600 text-white"
                : "bg-accent hover:bg-accent/90 text-accent-foreground"
            )}
          >
            {isSaving ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : saveSuccess ? (
              <>
                <Check className="mr-2 h-4 w-4" />
                Saved
              </>
            ) : (
              "Save Changes"
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}
