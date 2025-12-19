import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import { tauriCommands, tauriEvents } from "@/lib/tauri";
import { Mic, Activity, Lightbulb, Check } from "lucide-react";
import type { Preferences, MicrophoneDevice } from "@/types";

interface AudioTabProps {
  preferences: Preferences;
  onUpdate: <K extends keyof Preferences>(key: K, value: Preferences[K]) => void;
}

export function AudioTab({ preferences, onUpdate }: AudioTabProps) {
  const [devices, setDevices] = useState<MicrophoneDevice[]>([]);
  const [audioLevel, setAudioLevel] = useState(0);
  const [isLoading, setIsLoading] = useState(true);

  // Load available microphones
  useEffect(() => {
    const loadDevices = async () => {
      try {
        const mics = await tauriCommands.getMicrophones();
        setDevices(mics);
      } catch (err) {
        console.error("Failed to load microphones:", err);
      } finally {
        setIsLoading(false);
      }
    };
    loadDevices();
  }, []);

  // Subscribe to audio level updates
  useEffect(() => {
    let cleanup: (() => void) | undefined;

    tauriEvents.onAudioLevel((level) => {
      setAudioLevel(level);
    }).then((unsub) => {
      cleanup = unsub;
    });

    return () => cleanup?.();
  }, []);

  const selectedDevice = devices.find(d => d.id === preferences.microphone);

  return (
    <div className="space-y-8">
      {/* Microphone Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Mic className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Input Device</h2>
        </div>

        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5 space-y-4">
          <div className="space-y-2">
            <Label htmlFor="microphone" className="text-sm font-medium">Microphone</Label>
            <Select
              value={preferences.microphone || "default"}
              onValueChange={(value) => onUpdate("microphone", value)}
              disabled={isLoading}
            >
              <SelectTrigger id="microphone" className="w-full bg-white/[0.03] border-white/[0.08] hover:bg-white/[0.05] transition-colors">
                <SelectValue placeholder={isLoading ? "Loading..." : "Select microphone"} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="default">System Default</SelectItem>
                {devices.map((device) => (
                  <SelectItem key={device.id} value={device.id}>
                    {device.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {selectedDevice && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Check className="h-3 w-3 text-emerald-400" />
                Using: {selectedDevice.name}
              </div>
            )}
          </div>
        </div>
      </section>

      {/* Audio Level Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Activity className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Input Level</h2>
        </div>

        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5 space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Level Monitor</span>
            <span className="rounded-md bg-white/[0.06] px-2 py-1 text-xs font-medium tabular-nums">
              {Math.round(audioLevel * 100)}%
            </span>
          </div>

          {/* Visual meter - more sophisticated design */}
          <div className="relative">
            <div className="flex h-4 gap-[2px] overflow-hidden rounded-lg bg-white/[0.03]">
              {Array.from({ length: 24 }).map((_, i) => {
                const threshold = (i + 1) / 24;
                const isActive = audioLevel >= threshold;
                const isWarning = threshold > 0.75;
                const isPeak = threshold > 0.9;

                return (
                  <div
                    key={i}
                    className={cn(
                      "flex-1 rounded-sm transition-all duration-[50ms]",
                      isActive
                        ? isPeak
                          ? "bg-gradient-to-t from-rose-600 to-rose-400"
                          : isWarning
                          ? "bg-gradient-to-t from-amber-600 to-amber-400"
                          : "bg-gradient-to-t from-emerald-600 to-emerald-400"
                        : "bg-white/[0.04]"
                    )}
                  />
                );
              })}
            </div>
            {/* Level labels */}
            <div className="flex justify-between mt-2 text-[10px] text-muted-foreground">
              <span>Quiet</span>
              <span>Optimal</span>
              <span>Loud</span>
            </div>
          </div>

          <div className="rounded-lg bg-white/[0.03] p-3">
            <p className="text-xs text-muted-foreground leading-relaxed">
              Speak to test your microphone. The meter should stay in the <span className="text-emerald-400 font-medium">green zone</span> while talking.
            </p>
          </div>
        </div>
      </section>

      {/* Tips Section */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Lightbulb className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Tips</h2>
        </div>

        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5">
          <ul className="space-y-3">
            {[
              "Position your microphone 6-12 inches from your mouth",
              "Choose a quiet environment with minimal background noise",
              "Speak clearly at a consistent volume",
              "The level meter should stay in the green zone while speaking",
            ].map((tip, i) => (
              <li key={i} className="flex items-start gap-3 text-sm text-muted-foreground">
                <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-accent/10 text-[10px] font-semibold text-accent">
                  {i + 1}
                </span>
                <span className="leading-relaxed">{tip}</span>
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  );
}
