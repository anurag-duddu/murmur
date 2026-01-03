import { useState, useEffect, useCallback } from "react";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LanguageChips } from "@/components/shared/LanguageChips";
import { LanguageGrid } from "@/components/shared/LanguageGrid";
import { ChevronDown, ChevronUp, Keyboard, Eye, Volume2, Globe, Languages, User, LogOut } from "lucide-react";
import type { Preferences } from "@/types";
import { TRANSCRIPTION_LANGUAGES, HOTKEY_OPTIONS } from "@/types";
import type { UserInfo } from "@/types/auth";
import { tauriCommands } from "@/lib/tauri";

interface GeneralTabProps {
  preferences: Preferences;
  onUpdate: <K extends keyof Preferences>(key: K, value: Preferences[K]) => void;
}

export function GeneralTab({ preferences, onUpdate }: GeneralTabProps) {
  const [showLanguageEditor, setShowLanguageEditor] = useState(false);
  const [user, setUser] = useState<UserInfo | null>(null);
  const [isLoadingUser, setIsLoadingUser] = useState(true);
  const [isSigningOut, setIsSigningOut] = useState(false);

  // Fetch user info
  const fetchUser = useCallback(async () => {
    setIsLoadingUser(true);
    try {
      const userInfo = await tauriCommands.getUserInfo();
      setUser(userInfo);
    } catch (err) {
      console.error("[GeneralTab] Failed to get user info:", err);
    } finally {
      setIsLoadingUser(false);
    }
  }, []);

  // Fetch on mount
  useEffect(() => {
    fetchUser();
  }, [fetchUser]);

  // Re-fetch when window becomes visible (handles sign-in flow)
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        fetchUser();
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => document.removeEventListener("visibilitychange", handleVisibilityChange);
  }, [fetchUser]);

  const handleSignOut = useCallback(async () => {
    setIsSigningOut(true);

    // Safety timeout - reset after 10 seconds max in case of hang
    const timeout = setTimeout(() => {
      console.warn("[GeneralTab] Sign out timeout - resetting state");
      setIsSigningOut(false);
    }, 10000);

    try {
      await tauriCommands.logout();
      // The backend will handle showing the login window
      // Reset state for safety (window may hide before this runs)
      setIsSigningOut(false);
    } catch (err) {
      console.error("Failed to sign out:", err);
      setIsSigningOut(false);
    } finally {
      clearTimeout(timeout);
    }
  }, []);

  return (
    <div className="space-y-8">
      {/* Account Section - Always show, with loading state */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <User className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-white/50">Account</h2>
        </div>

        <div className="rounded-2xl glass-card p-4 sm:p-5">
          {isLoadingUser ? (
            // Loading skeleton
            <div className="flex items-center justify-between animate-pulse">
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-full bg-white/10" />
                <div className="space-y-2">
                  <div className="h-4 w-32 rounded bg-white/10" />
                  <div className="h-3 w-40 rounded bg-white/10" />
                </div>
              </div>
              <div className="h-8 w-24 rounded bg-white/10" />
            </div>
          ) : user ? (
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                {user.profile_picture_url ? (
                  <img
                    src={user.profile_picture_url}
                    alt="Profile"
                    className="h-10 w-10 rounded-full object-cover border border-white/10"
                  />
                ) : (
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent/20 border border-accent/30">
                    <User className="h-5 w-5 text-accent" />
                  </div>
                )}
                <div>
                  <p className="text-sm font-medium text-white">
                    {user.first_name && user.last_name
                      ? `${user.first_name} ${user.last_name}`
                      : user.email}
                  </p>
                  {user.first_name && user.last_name && (
                    <p className="text-xs text-white/50">{user.email}</p>
                  )}
                </div>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={handleSignOut}
                disabled={isSigningOut}
                className="gap-2 glass-input hover:bg-red-500/10 hover:border-red-500/30 hover:text-red-400"
              >
                <LogOut className="h-4 w-4" />
                {isSigningOut ? "Signing out..." : "Sign Out"}
              </Button>
            </div>
          ) : (
            // No user found - try to sign in again
            <div className="flex flex-col items-center gap-3 py-2">
              <p className="text-sm text-white/50">Session expired or not signed in</p>
              <Button
                variant="outline"
                size="sm"
                onClick={handleSignOut}
                disabled={isSigningOut}
                className="gap-2 glass-input"
              >
                {isSigningOut ? "Redirecting..." : "Sign In Again"}
              </Button>
            </div>
          )}
        </div>
      </section>

      {/* Recording Controls Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Keyboard className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-white/50">Recording Controls</h2>
        </div>

        <div className="space-y-4 rounded-2xl glass-card p-4 sm:p-5">
          {/* Recording Mode */}
          <div className="space-y-2">
            <Label htmlFor="recording-mode" className="text-sm font-medium text-white">Recording Mode</Label>
            <Select
              value={preferences.recording_mode}
              onValueChange={(value) =>
                onUpdate("recording_mode", value as "push-to-talk" | "toggle")
              }
            >
              <SelectTrigger id="recording-mode" className="w-full glass-input">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="push-to-talk">Push-to-talk (hold key)</SelectItem>
                <SelectItem value="toggle">Toggle (tap to start/stop)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Global Hotkey */}
          <div className="space-y-2">
            <Label htmlFor="hotkey" className="text-sm font-medium text-white">Global Hotkey</Label>
            <div className="flex gap-2">
              <Select
                value={preferences.hotkey}
                onValueChange={(value) => onUpdate("hotkey", value)}
              >
                <SelectTrigger id="hotkey" className="flex-1 glass-input">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {HOTKEY_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button variant="outline" size="default" className="shrink-0 glass-input hover:bg-white/[0.08]">
                Test
              </Button>
            </div>
          </div>
        </div>
      </section>

      {/* Feedback Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Eye className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-white/50">Feedback</h2>
        </div>

        <div className="rounded-2xl glass-card divide-y divide-white/[0.08]">
          <div className="flex items-center justify-between p-4 sm:p-5">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.06] border border-white/[0.08]">
                <Eye className="h-4 w-4 text-white/60" />
              </div>
              <div>
                <Label htmlFor="show-indicator" className="cursor-pointer text-sm font-medium text-white">
                  Visual Indicator
                </Label>
                <p className="text-xs text-white/50 mt-0.5">Show overlay when recording</p>
              </div>
            </div>
            <Switch
              id="show-indicator"
              checked={preferences.show_indicator}
              onCheckedChange={(checked) => onUpdate("show_indicator", checked)}
            />
          </div>
          <div className="flex items-center justify-between p-4 sm:p-5">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.06] border border-white/[0.08]">
                <Volume2 className="h-4 w-4 text-white/60" />
              </div>
              <div>
                <Label htmlFor="play-sounds" className="cursor-pointer text-sm font-medium text-white">
                  Sound Effects
                </Label>
                <p className="text-xs text-white/50 mt-0.5">Play audio feedback</p>
              </div>
            </div>
            <Switch
              id="play-sounds"
              checked={preferences.play_sounds}
              onCheckedChange={(checked) => onUpdate("play_sounds", checked)}
            />
          </div>
        </div>
      </section>

      {/* Language Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Globe className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-white/50">Language</h2>
        </div>

        {/* Transcription Language */}
        <div className="space-y-3 rounded-2xl glass-card p-4 sm:p-5">
          <Label htmlFor="language" className="text-sm font-medium text-white">Transcription Language</Label>
          <Select
            value={preferences.language}
            onValueChange={(value) => onUpdate("language", value)}
          >
            <SelectTrigger id="language" className="w-full glass-input">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="max-h-[300px]">
              {TRANSCRIPTION_LANGUAGES.map((lang) => (
                <SelectItem key={lang.value} value={lang.value}>
                  {lang.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <div className="rounded-lg bg-white/[0.04] border border-white/[0.06] p-3 text-xs text-white/50 space-y-1">
            <p><span className="text-white/80 font-medium">Native mode:</span> Select a specific language for strict transcription.</p>
            <p><span className="text-white/80 font-medium">Mixed mode:</span> Auto-detects among your spoken languages.</p>
          </div>
        </div>

        {/* Spoken Languages */}
        <div className="space-y-4 rounded-2xl glass-card p-4 sm:p-5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Languages className="h-4 w-4 text-white/60" />
              <Label className="text-sm font-medium text-white">Languages I Speak</Label>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowLanguageEditor(!showLanguageEditor)}
              className="h-8 gap-1.5 text-xs text-white/50 hover:text-white hover:bg-white/[0.06]"
            >
              {showLanguageEditor ? "Done" : "Edit"}
              {showLanguageEditor ? (
                <ChevronUp className="h-3 w-3" />
              ) : (
                <ChevronDown className="h-3 w-3" />
              )}
            </Button>
          </div>
          <LanguageChips languages={preferences.spoken_languages || ["en"]} />
          {showLanguageEditor && (
            <div className="mt-2 pt-4 border-t border-white/[0.08] space-y-4">
              <LanguageGrid
                selected={preferences.spoken_languages || ["en"]}
                onChange={(languages) => onUpdate("spoken_languages", languages)}
                compact
              />
              <Button
                size="sm"
                onClick={() => setShowLanguageEditor(false)}
                className="w-full bg-accent hover:bg-accent/90 text-accent-foreground"
              >
                Done
              </Button>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
