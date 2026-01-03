import { useState, useEffect, useCallback, useRef } from "react";
import { Button } from "@/components/ui/button";
import { PermissionCard } from "./PermissionCard";
import { StepIndicator } from "./StepIndicator";
import { MicrophoneSelector } from "./MicrophoneSelector";
import { SuccessScreen } from "./SuccessScreen";
import { tauriCommands } from "@/lib/tauri";
import { motion, AnimatePresence } from "framer-motion";
import { listen } from "@tauri-apps/api/event";

type PermissionStatus = "granted" | "denied" | "pending" | "checking";

export function OnboardingWindow() {
  const [currentStep, setCurrentStep] = useState(1);
  const [micStatus, setMicStatus] = useState<PermissionStatus>("checking");
  const [accessibilityStatus, setAccessibilityStatus] = useState<PermissionStatus>("checking");
  const [selectedMic, setSelectedMic] = useState("default");
  const [isRequestingMic, setIsRequestingMic] = useState(false);
  const [hotkey, setHotkey] = useState("Option + Space");
  const [isReauthorization, setIsReauthorization] = useState(false);

  // Track if user has clicked "Open System Settings" for accessibility
  const [isAwaitingAccessibility, setIsAwaitingAccessibility] = useState(false);
  // Track time since waiting for accessibility (for restart button fallback)
  const [accessibilityWaitTime, setAccessibilityWaitTime] = useState(0);

  // Track if we've started checking permissions
  const hasStartedRef = useRef(false);

  // Check permissions when we receive the "start-onboarding" event from backend
  // OR as a fallback, check on mount after a delay (handles race condition where
  // the event might be emitted before the listener is set up)
  useEffect(() => {
    let interval: ReturnType<typeof setInterval> | null = null;
    let unlistenEvent: (() => void) | null = null;
    let mountTimeout: ReturnType<typeof setTimeout> | null = null;

    const checkPermissions = async () => {
      try {
        // Check if this is a re-authorization scenario (app rebuilt/reinstalled)
        const needsReauth = await tauriCommands.needsReauthorization();
        setIsReauthorization(needsReauth);

        const status = await tauriCommands.checkPermissions();
        // microphone is a string: "granted", "denied", or "undetermined"
        setMicStatus(status.microphone === "granted" ? "granted" : "pending");
        // accessibility is a boolean
        setAccessibilityStatus(status.accessibility === true ? "granted" : "pending");
      } catch (err) {
        console.error("Failed to check permissions:", err);
        setMicStatus("pending");
        setAccessibilityStatus("pending");
      }
    };

    const startPolling = () => {
      if (hasStartedRef.current) return;
      hasStartedRef.current = true;

      console.log("[Onboarding] Starting permission checks");

      // Initial check
      checkPermissions();

      // Poll for BOTH permissions (user may grant in System Settings)
      interval = setInterval(async () => {
        try {
          const status = await tauriCommands.checkPermissions();
          // Update both permissions - fixes asymmetry where only accessibility was polled
          setMicStatus(status.microphone === "granted" ? "granted" : "pending");
          setAccessibilityStatus(status.accessibility === true ? "granted" : "pending");
        } catch (err) {
          // Ignore polling errors
        }
      }, 1000);
    };

    const stopPolling = () => {
      if (interval) {
        clearInterval(interval);
        interval = null;
      }
    };

    const setup = async () => {
      // Listen for explicit "start-onboarding" event from backend (OAuth callback flow)
      unlistenEvent = await listen("start-onboarding", () => {
        console.log("[Onboarding] Received start-onboarding event from backend");
        startPolling();
      });

      // FALLBACK: Also start polling after a short delay on mount
      // This handles the race condition where the backend emits the event
      // before the React listener is set up (e.g., on direct app startup)
      mountTimeout = setTimeout(() => {
        if (!hasStartedRef.current) {
          console.log("[Onboarding] Starting permission checks (mount fallback)");
          startPolling();
        }
      }, 300); // 300ms fallback - enough time for the event to arrive if it was sent
    };

    setup();

    return () => {
      stopPolling();
      if (unlistenEvent) unlistenEvent();
      if (mountTimeout) clearTimeout(mountTimeout);
    };
  }, []);

  // Load hotkey from preferences
  useEffect(() => {
    const loadHotkey = async () => {
      try {
        const prefs = await tauriCommands.getPreferences();
        if (prefs.hotkey) {
          // Format hotkey for display
          const formatted = prefs.hotkey
            .replace("CmdOrCtrl", "Cmd")
            .replace("+", " + ");
          setHotkey(formatted);
        }
      } catch (err) {
        // Use default
      }
    };
    loadHotkey();
  }, []);

  // Track how long we've been waiting for accessibility permission
  useEffect(() => {
    let timer: ReturnType<typeof setInterval> | null = null;

    if (isAwaitingAccessibility && accessibilityStatus !== "granted") {
      timer = setInterval(() => {
        setAccessibilityWaitTime((t) => t + 1);
      }, 1000);
    } else if (accessibilityStatus === "granted") {
      // Reset when granted
      setAccessibilityWaitTime(0);
      setIsAwaitingAccessibility(false);
    }

    return () => {
      if (timer) clearInterval(timer);
    };
  }, [isAwaitingAccessibility, accessibilityStatus]);

  // Request microphone permission
  const handleRequestMic = useCallback(async () => {
    setIsRequestingMic(true);
    try {
      const granted = await tauriCommands.requestMicrophonePermission();
      setMicStatus(granted ? "granted" : "denied");
    } catch (err) {
      console.error("Failed to request microphone:", err);
      setMicStatus("denied");
    } finally {
      setIsRequestingMic(false);
    }
  }, []);

  // Request accessibility permission (shows system dialog)
  const handleRequestAccessibility = useCallback(async () => {
    try {
      // This triggers macOS to show a dialog directing user to System Settings
      const granted = await tauriCommands.requestAccessibilityPermission();
      if (granted) {
        setAccessibilityStatus("granted");
      } else {
        // Start waiting for user to grant in System Settings
        setIsAwaitingAccessibility(true);
        setAccessibilityWaitTime(0);
      }
      // If not granted, the system dialog will guide user to System Settings
      // Permission status will be updated by the polling
    } catch (err) {
      console.error("Failed to request accessibility:", err);
      // Fallback to opening settings directly
      setIsAwaitingAccessibility(true);
      setAccessibilityWaitTime(0);
      await tauriCommands.openAccessibilitySettings();
    }
  }, []);

  // Force immediate re-check of accessibility permission
  const handleCheckAccessibilityNow = useCallback(async () => {
    try {
      const status = await tauriCommands.checkPermissions();
      setAccessibilityStatus(status.accessibility === true ? "granted" : "pending");
    } catch (err) {
      console.error("Failed to check accessibility:", err);
    }
  }, []);

  // Restart the app (for edge cases where accessibility needs restart)
  const handleRestartApp = useCallback(async () => {
    try {
      await tauriCommands.restartApp();
    } catch (err) {
      console.error("Failed to restart app:", err);
    }
  }, []);

  // Continue to next step
  const handleContinue = useCallback(() => {
    setCurrentStep(2);
  }, []);

  // Finish onboarding
  const handleFinish = useCallback(async () => {
    try {
      // Save selected microphone
      const prefs = await tauriCommands.getPreferences();
      await tauriCommands.updatePreferences({
        ...prefs,
        microphone: selectedMic,
        onboarding_complete: true,
      });
      // Close onboarding window
      await tauriCommands.completeOnboarding();
    } catch (err) {
      console.error("Failed to complete onboarding:", err);
    }
  }, [selectedMic]);

  const canContinue = micStatus === "granted" && accessibilityStatus === "granted";

  return (
    <div className="flex min-h-screen flex-col items-center justify-center glass-window p-10 relative overflow-hidden">
      {/* Ambient background glow */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-96 h-96 bg-accent/10 rounded-full blur-[100px] pointer-events-none" />
      <div className="absolute bottom-1/4 right-1/4 w-64 h-64 bg-blue-500/8 rounded-full blur-[80px] pointer-events-none" />

      <div className="relative w-full max-w-md">
        {/* Re-authorization banner */}
        {isReauthorization && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-6 rounded-lg bg-amber-500/20 border border-amber-500/30 p-4"
          >
            <p className="text-sm text-amber-200">
              <span className="font-medium">App was updated.</span>{" "}
              Please re-grant Accessibility permission to enable text insertion.
            </p>
          </motion.div>
        )}

        {/* Logo */}
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-bold text-white">Keyhold</h1>
          <p className="mt-2 text-white/50">
            {isReauthorization ? "Quick permission refresh" : "Voice to text, effortlessly"}
          </p>
        </div>

        {/* Step Indicator */}
        <div className="mb-8">
          <StepIndicator currentStep={currentStep} totalSteps={2} />
        </div>

        {/* Content */}
        <AnimatePresence mode="wait">
          {currentStep === 1 ? (
            <motion.div
              key="step1"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20 }}
              transition={{ duration: 0.3 }}
              className="space-y-4"
            >
              {/* Permission Cards */}
              <PermissionCard
                type="microphone"
                status={micStatus}
                onRequest={handleRequestMic}
                isRequesting={isRequestingMic}
              />

              <PermissionCard
                type="accessibility"
                status={accessibilityStatus}
                onRequest={handleRequestAccessibility}
                isAwaitingGrant={isAwaitingAccessibility}
                onCheckNow={handleCheckAccessibilityNow}
              />

              {/* Restart App button - shown after 30 seconds of waiting for accessibility */}
              {isAwaitingAccessibility && accessibilityWaitTime >= 30 && accessibilityStatus !== "granted" && (
                <motion.div
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="rounded-lg bg-amber-500/10 border border-amber-500/20 p-4"
                >
                  <p className="text-sm text-amber-200 mb-3">
                    Still waiting? Some systems require an app restart after granting accessibility.
                  </p>
                  <Button
                    onClick={handleRestartApp}
                    variant="outline"
                    className="w-full bg-amber-500/20 text-amber-200 hover:bg-amber-500/30 border-amber-500/30"
                  >
                    Restart Keyhold
                  </Button>
                </motion.div>
              )}

              {/* Microphone Selector (shown when mic permission granted) */}
              {micStatus === "granted" && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: "auto" }}
                  transition={{ duration: 0.3 }}
                >
                  <MicrophoneSelector
                    value={selectedMic}
                    onChange={setSelectedMic}
                  />
                </motion.div>
              )}

              {/* Continue Button */}
              <div className="pt-4">
                <Button
                  onClick={handleContinue}
                  disabled={!canContinue}
                  className="w-full"
                  size="lg"
                >
                  Continue
                </Button>
              </div>
            </motion.div>
          ) : (
            <motion.div
              key="step2"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ duration: 0.3 }}
            >
              <SuccessScreen hotkey={hotkey} onFinish={handleFinish} />
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
