import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Mic, Accessibility, Check, Loader2 } from "lucide-react";

type PermissionType = "microphone" | "accessibility";
type PermissionStatus = "granted" | "denied" | "pending" | "checking";

interface PermissionCardProps {
  type: PermissionType;
  status: PermissionStatus;
  onRequest: () => void;
  isRequesting?: boolean;
}

const PERMISSION_CONFIG = {
  microphone: {
    icon: Mic,
    title: "Microphone Access",
    description: "Required to capture your voice",
    buttonText: "Grant Microphone Access",
    iconBg: "bg-gradient-to-br from-red-500 to-red-600",
  },
  accessibility: {
    icon: Accessibility,
    title: "Accessibility Access",
    description: "Required for auto-paste feature",
    buttonText: "Open System Settings",
    iconBg: "bg-gradient-to-br from-primary to-blue-600",
  },
};

const STATUS_CONFIG = {
  granted: { text: "Granted", dotClass: "bg-success" },
  denied: { text: "Denied", dotClass: "bg-destructive" },
  pending: { text: "Not granted", dotClass: "bg-warning" },
  checking: { text: "Checking...", dotClass: "bg-warning" },
};

export function PermissionCard({
  type,
  status,
  onRequest,
  isRequesting = false,
}: PermissionCardProps) {
  const config = PERMISSION_CONFIG[type];
  const statusConfig = STATUS_CONFIG[status];
  const Icon = config.icon;
  const isGranted = status === "granted";

  return (
    <div className="rounded-2xl border border-white/10 bg-white/[0.08] p-6">
      <div className="flex items-center gap-4">
        {/* Icon */}
        <div
          className={cn(
            "flex h-12 w-12 items-center justify-center rounded-xl",
            config.iconBg
          )}
        >
          <Icon className="h-6 w-6 text-white" />
        </div>

        {/* Title & Description */}
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-white">{config.title}</h3>
          <p className="text-sm text-white/60">{config.description}</p>
        </div>

        {/* Status */}
        <div className="flex items-center gap-2 text-sm font-medium">
          <span className={cn("h-2.5 w-2.5 rounded-full", statusConfig.dotClass)} />
          <span className="text-white/80">{statusConfig.text}</span>
        </div>
      </div>

      {/* Action Button */}
      {!isGranted && (
        <div className="mt-4">
          <Button
            onClick={onRequest}
            disabled={isRequesting || status === "checking"}
            className={cn(
              "w-full",
              type === "accessibility" &&
                "bg-white/10 text-white hover:bg-white/15 border border-white/20"
            )}
          >
            {isRequesting || status === "checking" ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : isGranted ? (
              <Check className="mr-2 h-4 w-4" />
            ) : null}
            {isGranted ? "Granted" : config.buttonText}
          </Button>
        </div>
      )}

      {isGranted && (
        <div className="mt-4 flex items-center justify-center gap-2 rounded-lg bg-success/10 py-2 text-sm text-success">
          <Check className="h-4 w-4" />
          Permission granted
        </div>
      )}
    </div>
  );
}
