import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { tauriCommands } from "@/lib/tauri";
import type { Preferences } from "@/types";
import { DEFAULT_PREFERENCES } from "@/types";

/**
 * Deep equality check for Preferences objects.
 * More reliable than JSON.stringify which can have key ordering issues.
 */
function preferencesEqual(a: Preferences, b: Preferences): boolean {
  // Check primitive fields
  if (
    a.recording_mode !== b.recording_mode ||
    a.hotkey !== b.hotkey ||
    a.show_indicator !== b.show_indicator ||
    a.play_sounds !== b.play_sounds ||
    a.microphone !== b.microphone ||
    a.language !== b.language ||
    a.deepgram_api_key !== b.deepgram_api_key ||
    a.groq_api_key !== b.groq_api_key ||
    a.anthropic_api_key !== b.anthropic_api_key ||
    a.transcription_provider !== b.transcription_provider ||
    a.license_key !== b.license_key ||
    a.onboarding_complete !== b.onboarding_complete
  ) {
    return false;
  }

  // Check spoken_languages array
  if (a.spoken_languages.length !== b.spoken_languages.length) {
    return false;
  }
  for (let i = 0; i < a.spoken_languages.length; i++) {
    if (a.spoken_languages[i] !== b.spoken_languages[i]) {
      return false;
    }
  }

  return true;
}

export function usePreferences() {
  const [preferences, setPreferences] = useState<Preferences>(DEFAULT_PREFERENCES);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Track original preferences to detect changes
  const originalPrefsRef = useRef<Preferences>(DEFAULT_PREFERENCES);

  // Load preferences on mount
  useEffect(() => {
    const load = async () => {
      try {
        const prefs = await tauriCommands.getPreferences();
        const merged = { ...DEFAULT_PREFERENCES, ...prefs };
        setPreferences(merged);
        originalPrefsRef.current = merged;
        setError(null);
      } catch (err) {
        console.error("Failed to load preferences:", err);
        setError(String(err));
      } finally {
        setIsLoading(false);
      }
    };
    load();
  }, []);

  // Check if preferences have changed using proper deep comparison
  const hasChanges = useMemo(
    () => !preferencesEqual(preferences, originalPrefsRef.current),
    [preferences]
  );

  // Update a single preference
  const updatePreference = useCallback(
    <K extends keyof Preferences>(key: K, value: Preferences[K]) => {
      setPreferences((prev) => ({ ...prev, [key]: value }));
    },
    []
  );

  // Save all preferences
  const savePreferences = useCallback(async () => {
    setIsSaving(true);
    setError(null);
    try {
      await tauriCommands.updatePreferences(preferences);
      originalPrefsRef.current = preferences; // Update original after successful save
      return true;
    } catch (err) {
      console.error("Failed to save preferences:", err);
      setError(String(err));
      return false;
    } finally {
      setIsSaving(false);
    }
  }, [preferences]);

  // Reset to original preferences (discard changes)
  const resetPreferences = useCallback(() => {
    setPreferences(originalPrefsRef.current);
  }, []);

  // Save specific updates (for auto-save scenarios)
  const saveUpdates = useCallback(async (updates: Partial<Preferences>) => {
    const newPrefs = { ...preferences, ...updates };
    setPreferences(newPrefs);
    setIsSaving(true);
    try {
      await tauriCommands.updatePreferences(newPrefs);
      return true;
    } catch (err) {
      console.error("Failed to save preferences:", err);
      return false;
    } finally {
      setIsSaving(false);
    }
  }, [preferences]);

  return {
    preferences,
    isLoading,
    isSaving,
    error,
    hasChanges,
    updatePreference,
    savePreferences,
    resetPreferences,
    saveUpdates,
    setPreferences,
  };
}
