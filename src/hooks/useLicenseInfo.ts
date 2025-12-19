import { useState, useEffect, useCallback } from "react";
import { tauriCommands } from "@/lib/tauri";
import type { LicenseInfo } from "@/types";
import { DEFAULT_LICENSE_INFO } from "@/types";

export function useLicenseInfo() {
  const [license, setLicense] = useState<LicenseInfo>(DEFAULT_LICENSE_INFO);
  const [isLoading, setIsLoading] = useState(true);
  const [isActivating, setIsActivating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadLicense = useCallback(async () => {
    try {
      const result = await tauriCommands.getLicenseInfo();
      setLicense(result);
      setError(null);
    } catch (err) {
      console.error("Failed to load license info:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    loadLicense();
  }, [loadLicense]);

  const activateLicense = useCallback(async (licenseKey: string) => {
    setIsActivating(true);
    setError(null);
    try {
      const result = await tauriCommands.activateLicense(licenseKey);
      setLicense(result);
      if (!result.valid) {
        setError("Invalid license key");
      }
      return result;
    } catch (err) {
      const errorMsg = String(err);
      console.error("Failed to activate license:", err);
      setError(errorMsg);
      return null;
    } finally {
      setIsActivating(false);
    }
  }, []);

  const clearLicense = useCallback(async () => {
    try {
      await tauriCommands.clearLicense();
      setLicense(DEFAULT_LICENSE_INFO);
    } catch (err) {
      console.error("Failed to clear license:", err);
    }
  }, []);

  return {
    license,
    isLoading,
    isActivating,
    error,
    isValid: license.valid,
    tier: license.tier,
    activateLicense,
    clearLicense,
    refresh: loadLicense,
  };
}
