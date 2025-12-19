import { useState } from "react";
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
import { ChevronDown, ChevronUp, Keyboard, Eye, Volume2, Globe, Languages } from "lucide-react";
import type { Preferences } from "@/types";
import { TRANSCRIPTION_LANGUAGES, HOTKEY_OPTIONS } from "@/types";

interface GeneralTabProps {
  preferences: Preferences;
  onUpdate: <K extends keyof Preferences>(key: K, value: Preferences[K]) => void;
}

export function GeneralTab({ preferences, onUpdate }: GeneralTabProps) {
  const [showLanguageEditor, setShowLanguageEditor] = useState(false);

  return (
    <div className="space-y-8">
      {/* Recording Controls Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Keyboard className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Recording Controls</h2>
        </div>

        <div className="space-y-4 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5">
          {/* Recording Mode */}
          <div className="space-y-2">
            <Label htmlFor="recording-mode" className="text-sm font-medium">Recording Mode</Label>
            <Select
              value={preferences.recording_mode}
              onValueChange={(value) =>
                onUpdate("recording_mode", value as "push-to-talk" | "toggle")
              }
            >
              <SelectTrigger id="recording-mode" className="w-full bg-white/[0.03] border-white/[0.08] hover:bg-white/[0.05] transition-colors">
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
            <Label htmlFor="hotkey" className="text-sm font-medium">Global Hotkey</Label>
            <div className="flex gap-2">
              <Select
                value={preferences.hotkey}
                onValueChange={(value) => onUpdate("hotkey", value)}
              >
                <SelectTrigger id="hotkey" className="flex-1 bg-white/[0.03] border-white/[0.08] hover:bg-white/[0.05] transition-colors">
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
              <Button variant="outline" size="default" className="shrink-0 bg-white/[0.03] border-white/[0.08] hover:bg-white/[0.06]">
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
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Feedback</h2>
        </div>

        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] divide-y divide-white/[0.06]">
          <div className="flex items-center justify-between p-4 sm:p-5">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.04]">
                <Eye className="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <Label htmlFor="show-indicator" className="cursor-pointer text-sm font-medium">
                  Visual Indicator
                </Label>
                <p className="text-xs text-muted-foreground mt-0.5">Show overlay when recording</p>
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
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.04]">
                <Volume2 className="h-4 w-4 text-muted-foreground" />
              </div>
              <div>
                <Label htmlFor="play-sounds" className="cursor-pointer text-sm font-medium">
                  Sound Effects
                </Label>
                <p className="text-xs text-muted-foreground mt-0.5">Play audio feedback</p>
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
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Language</h2>
        </div>

        {/* Transcription Language */}
        <div className="space-y-3 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5">
          <Label htmlFor="language" className="text-sm font-medium">Transcription Language</Label>
          <Select
            value={preferences.language}
            onValueChange={(value) => onUpdate("language", value)}
          >
            <SelectTrigger id="language" className="w-full bg-white/[0.03] border-white/[0.08] hover:bg-white/[0.05] transition-colors">
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
          <div className="rounded-lg bg-white/[0.03] p-3 text-xs text-muted-foreground space-y-1">
            <p><span className="text-foreground font-medium">Native mode:</span> Select a specific language for strict transcription.</p>
            <p><span className="text-foreground font-medium">Mixed mode:</span> Auto-detects among your spoken languages.</p>
          </div>
        </div>

        {/* Spoken Languages */}
        <div className="space-y-4 rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Languages className="h-4 w-4 text-muted-foreground" />
              <Label className="text-sm font-medium">Languages I Speak</Label>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowLanguageEditor(!showLanguageEditor)}
              className="h-8 gap-1.5 text-xs text-muted-foreground hover:text-foreground"
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
            <div className="mt-2 pt-4 border-t border-white/[0.06] space-y-4">
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
