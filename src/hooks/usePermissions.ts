import { useState, useEffect, useCallback, useRef } from "react";
import { tauriCommands } from "@/lib/tauri";
import type { PermissionStatus, MicrophoneDevice } from "@/types";
import { DEFAULT_PERMISSION_STATUS } from "@/types";

interface UsePermissionsOptions {
  pollInterval?: number;
  pollOnFocus?: boolean;
}

export function usePermissions(options: UsePermissionsOptions = {}) {
  const { pollInterval = 3000, pollOnFocus = true } = options;

  const [status, setStatus] = useState<PermissionStatus>(DEFAULT_PERMISSION_STATUS);
  const [microphones, setMicrophones] = useState<MicrophoneDevice[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const pollRef = useRef<number | null>(null);

  const checkPermissions = useCallback(async () => {
    try {
      const result = await tauriCommands.checkPermissions();
      setStatus(result);

      // Load microphones if we have permission
      if (result.microphone === "granted") {
        const mics = await tauriCommands.getMicrophones();
        setMicrophones(mics);
      }
    } catch (err) {
      console.error("Failed to check permissions:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const requestMicrophonePermission = useCallback(async () => {
    try {
      await tauriCommands.requestMicrophonePermission();
      await checkPermissions();
    } catch (err) {
      console.error("Failed to request microphone permission:", err);
    }
  }, [checkPermissions]);

  const openAccessibilitySettings = useCallback(async () => {
    try {
      await tauriCommands.openAccessibilitySettings();
    } catch (err) {
      console.error("Failed to open accessibility settings:", err);
    }
  }, []);

  const setSelectedMicrophone = useCallback(async (deviceId: string) => {
    try {
      await tauriCommands.setSelectedMicrophone(deviceId);
    } catch (err) {
      console.error("Failed to set microphone:", err);
    }
  }, []);

  const refreshMicrophones = useCallback(async () => {
    try {
      const mics = await tauriCommands.getMicrophones();
      setMicrophones(mics);
    } catch (err) {
      console.error("Failed to refresh microphones:", err);
    }
  }, []);

  // Initial check and polling
  useEffect(() => {
    checkPermissions();

    // Set up polling
    pollRef.current = window.setInterval(checkPermissions, pollInterval);

    return () => {
      if (pollRef.current) {
        clearInterval(pollRef.current);
      }
    };
  }, [checkPermissions, pollInterval]);

  // Poll on window focus
  useEffect(() => {
    if (!pollOnFocus) return;

    const handleFocus = () => {
      checkPermissions();
    };

    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [checkPermissions, pollOnFocus]);

  return {
    status,
    microphones,
    isLoading,
    hasMicrophonePermission: status.microphone === "granted",
    hasAccessibilityPermission: status.accessibility,
    canContinue: status.microphone === "granted",
    checkPermissions,
    requestMicrophonePermission,
    openAccessibilitySettings,
    setSelectedMicrophone,
    refreshMicrophones,
  };
}
