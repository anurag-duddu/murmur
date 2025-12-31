import { useState, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { PermissionCard } from "./PermissionCard";
import { StepIndicator } from "./StepIndicator";
import { MicrophoneSelector } from "./MicrophoneSelector";
import { SuccessScreen } from "./SuccessScreen";
import { tauriCommands } from "@/lib/tauri";
import { motion, AnimatePresence } from "framer-motion";

type PermissionStatus = "granted" | "denied" | "pending" | "checking";

export function OnboardingWindow() {
  const [currentStep, setCurrentStep] = useState(1);
  const [micStatus, setMicStatus] = useState<PermissionStatus>("checking");
  const [accessibilityStatus, setAccessibilityStatus] = useState<PermissionStatus>("checking");
  const [selectedMic, setSelectedMic] = useState("default");
  const [isRequestingMic, setIsRequestingMic] = useState(false);
  const [hotkey, setHotkey] = useState("Option + Space");

  // Check permissions on mount
  useEffect(() => {
    const checkPermissions = async () => {
      try {
        const status = await tauriCommands.getPermissionStatus();
        setMicStatus(status.microphone ? "granted" : "pending");
        setAccessibilityStatus(status.accessibility ? "granted" : "pending");
      } catch (err) {
        console.error("Failed to check permissions:", err);
        setMicStatus("pending");
        setAccessibilityStatus("pending");
      }
    };
    checkPermissions();

    // Poll for accessibility status (user must grant in System Settings)
    const interval = setInterval(async () => {
      try {
        const status = await tauriCommands.getPermissionStatus();
        setAccessibilityStatus(status.accessibility ? "granted" : "pending");
      } catch (err) {
        // Ignore polling errors
      }
    }, 1000);

    return () => clearInterval(interval);
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

  // Open accessibility settings
  const handleOpenAccessibility = useCallback(async () => {
    try {
      await tauriCommands.openAccessibilitySettings();
    } catch (err) {
      console.error("Failed to open accessibility settings:", err);
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
    <div className="flex min-h-screen flex-col items-center justify-center bg-gradient-to-br from-[#1a1a2e] to-[#16213e] p-10">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-bold text-white">Murmur</h1>
          <p className="mt-2 text-white/60">Voice to text, effortlessly</p>
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
                onRequest={handleOpenAccessibility}
              />

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
