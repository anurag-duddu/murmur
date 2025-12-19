import { useEffect, useState } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Mic } from "lucide-react";
import { tauriCommands } from "@/lib/tauri";
import type { MicrophoneDevice } from "@/types";

interface MicrophoneSelectorProps {
  value: string;
  onChange: (value: string) => void;
}

export function MicrophoneSelector({ value, onChange }: MicrophoneSelectorProps) {
  const [devices, setDevices] = useState<MicrophoneDevice[]>([]);
  const [isLoading, setIsLoading] = useState(true);

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

  return (
    <div className="rounded-2xl border border-white/10 bg-white/[0.08] p-6">
      <Label className="mb-4 flex items-center gap-2 text-base font-semibold text-white">
        <Mic className="h-4 w-4" />
        Select Microphone
      </Label>

      <Select value={value} onValueChange={onChange} disabled={isLoading}>
        <SelectTrigger className="w-full border-white/20 bg-white/10 text-white">
          <SelectValue placeholder={isLoading ? "Loading..." : "Select microphone"} />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="default">Default Microphone</SelectItem>
          {devices.map((device) => (
            <SelectItem key={device.id} value={device.id}>
              {device.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
