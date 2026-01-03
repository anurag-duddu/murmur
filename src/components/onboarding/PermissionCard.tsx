import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Mic, Accessibility, Check, Loader2, RefreshCw, ExternalLink } from "lucide-react";
import { tauriCommands } from "@/lib/tauri";

type PermissionType = "microphone" | "accessibility";
type PermissionStatus = "granted" | "denied" | "pending" | "checking";

interface PermissionCardProps {
  type: PermissionType;
  status: PermissionStatus;
  onRequest: () => void;
  isRequesting?: boolean;
  // Accessibility-specific: waiting for user to grant in System Settings
  isAwaitingGrant?: boolean;
  onCheckNow?: () => void;
}

const PERMISSION_CONFIG = {
  microphone: {
    icon: Mic,
    title: "Microphone Access",
    description: "Required to capture your voice",
    buttonText: "Grant Microphone Access",
    iconBg: "bg-gradient-to-br from-accent to-orange-600 shadow-lg shadow-accent/25",
  },
  accessibility: {
    icon: Accessibility,
    title: "Accessibility Access",
    description: "Required for auto-paste feature",
    buttonText: "Open System Settings",
    iconBg: "bg-gradient-to-br from-blue-500 to-blue-600 shadow-lg shadow-blue-500/25",
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
  isAwaitingGrant = false,
  onCheckNow,
}: PermissionCardProps) {
  const config = PERMISSION_CONFIG[type];
  const statusConfig = STATUS_CONFIG[status];
  const Icon = config.icon;
  const isGranted = status === "granted";
  const isDenied = status === "denied";

  // For accessibility, show "waiting" state after user clicks "Open System Settings"
  const showWaitingState = type === "accessibility" && isAwaitingGrant && !isGranted;

  // For microphone, show "denied" recovery state with link to System Settings
  const showMicDeniedState = type === "microphone" && isDenied;

  const handleOpenMicrophoneSettings = async () => {
    try {
      await tauriCommands.openMicrophoneSettings();
    } catch (err) {
      console.error("Failed to open microphone settings:", err);
    }
  };

  return (
    <div className="rounded-2xl glass-card p-6">
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
          <p className="text-sm text-white/50">{config.description}</p>
        </div>

        {/* Status */}
        <div className="flex items-center gap-2 text-sm font-medium">
          {showWaitingState ? (
            <>
              <Loader2 className="h-3.5 w-3.5 animate-spin text-blue-400" />
              <span className="text-blue-400">Waiting...</span>
            </>
          ) : (
            <>
              <span className={cn("h-2.5 w-2.5 rounded-full shadow-sm", statusConfig.dotClass)} />
              <span className="text-white/70">{statusConfig.text}</span>
            </>
          )}
        </div>
      </div>

      {/* Waiting state for accessibility - shows after clicking "Open System Settings" */}
      {showWaitingState && (
        <div className="mt-4 space-y-3">
          <div className="rounded-lg bg-blue-500/10 border border-blue-500/20 p-3">
            <p className="text-sm text-blue-200">
              Grant permission in System Settings, then return here. Permission will be detected automatically.
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              onClick={onRequest}
              variant="outline"
              className="flex-1 bg-white/[0.05] text-white hover:bg-white/[0.08] border-white/[0.1]"
            >
              Open Settings Again
            </Button>
            {onCheckNow && (
              <Button
                onClick={onCheckNow}
                variant="outline"
                className="bg-white/[0.05] text-white hover:bg-white/[0.08] border-white/[0.1]"
              >
                <RefreshCw className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      )}

      {/* Denied state for microphone - shows button to open System Settings */}
      {showMicDeniedState && (
        <div className="mt-4 space-y-3">
          <div className="rounded-lg bg-destructive/10 border border-destructive/20 p-3">
            <p className="text-sm text-destructive">
              Microphone access was denied. Please enable it in System Settings to continue.
            </p>
          </div>
          <Button
            onClick={handleOpenMicrophoneSettings}
            className="w-full gap-2 bg-white/[0.08] text-white hover:bg-white/[0.12] border border-white/[0.15]"
          >
            <ExternalLink className="h-4 w-4" />
            Open Microphone Settings
          </Button>
        </div>
      )}

      {/* Normal action button (before waiting state or denied state) */}
      {!isGranted && !showWaitingState && !showMicDeniedState && (
        <div className="mt-4">
          <Button
            onClick={onRequest}
            disabled={isRequesting || status === "checking"}
            className={cn(
              "w-full transition-all duration-150",
              type === "accessibility"
                ? "bg-white/[0.08] text-white hover:bg-white/[0.12] border border-white/[0.15]"
                : "bg-accent hover:bg-accent/90 text-accent-foreground"
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
        <div className="mt-4 flex items-center justify-center gap-2 rounded-lg bg-success/15 border border-success/20 py-2 text-sm text-success">
          <Check className="h-4 w-4" />
          Permission granted
        </div>
      )}
    </div>
  );
}
