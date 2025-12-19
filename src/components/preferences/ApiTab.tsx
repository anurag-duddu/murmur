import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import { Eye, EyeOff, Check, X, Loader2, Key, Shield, Sparkles } from "lucide-react";
import { tauriCommands } from "@/lib/tauri";
import type { Preferences } from "@/types";

interface ApiTabProps {
  preferences: Preferences;
  onUpdate: <K extends keyof Preferences>(key: K, value: Preferences[K]) => void;
}

interface ApiKeyInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  helpText?: string;
  helpLink?: { text: string; url: string };
  testEndpoint?: () => Promise<boolean>;
}

function ApiKeyInput({
  label,
  value,
  onChange,
  placeholder,
  helpText,
  helpLink,
  testEndpoint,
}: ApiKeyInputProps) {
  const [showKey, setShowKey] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);

  const handleTest = async () => {
    if (!testEndpoint || !value.trim()) return;

    setIsTesting(true);
    setTestResult(null);

    try {
      const result = await testEndpoint();
      setTestResult(result);
    } catch {
      setTestResult(false);
    } finally {
      setIsTesting(false);
    }
  };

  return (
    <div className="space-y-3">
      <Label className="text-sm font-medium">{label}</Label>
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Input
            type={showKey ? "text" : "password"}
            value={value}
            onChange={(e) => {
              onChange(e.target.value);
              setTestResult(null);
            }}
            placeholder={placeholder}
            className="pr-10 bg-white/[0.03] border-white/[0.08] hover:bg-white/[0.05] focus:bg-white/[0.05] transition-colors"
          />
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="absolute right-1 top-1/2 h-7 w-7 -translate-y-1/2 p-0 hover:bg-white/[0.06]"
            onClick={() => setShowKey(!showKey)}
          >
            {showKey ? (
              <EyeOff className="h-4 w-4 text-muted-foreground" />
            ) : (
              <Eye className="h-4 w-4 text-muted-foreground" />
            )}
          </Button>
        </div>
        {testEndpoint && (
          <Button
            variant="outline"
            onClick={handleTest}
            disabled={isTesting || !value.trim()}
            className={cn(
              "min-w-[80px] bg-white/[0.03] border-white/[0.08] transition-all",
              testResult === true && "border-emerald-500/50 text-emerald-400 bg-emerald-500/10",
              testResult === false && "border-destructive/50 text-destructive bg-destructive/10"
            )}
          >
            {isTesting ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : testResult === true ? (
              <Check className="h-4 w-4" />
            ) : testResult === false ? (
              <X className="h-4 w-4" />
            ) : (
              "Test"
            )}
          </Button>
        )}
      </div>
      {(helpText || helpLink) && (
        <p className="text-xs text-muted-foreground">
          {helpText}
          {helpLink && (
            <a
              href={helpLink.url}
              target="_blank"
              rel="noopener noreferrer"
              className="ml-1 text-accent hover:underline"
            >
              {helpLink.text} →
            </a>
          )}
        </p>
      )}
      {testResult !== null && (
        <div
          className={cn(
            "flex items-center gap-2 rounded-lg px-3 py-2 text-xs",
            testResult ? "bg-emerald-500/10 text-emerald-400" : "bg-destructive/10 text-destructive"
          )}
        >
          {testResult ? (
            <>
              <Check className="h-3.5 w-3.5" />
              Connection successful!
            </>
          ) : (
            <>
              <X className="h-3.5 w-3.5" />
              Connection failed. Please check your API key.
            </>
          )}
        </div>
      )}
    </div>
  );
}

export function ApiTab({ preferences, onUpdate }: ApiTabProps) {
  const testDeepgram = async () => {
    try {
      return await tauriCommands.testDeepgramKey(preferences.deepgram_api_key || "");
    } catch {
      return false;
    }
  };

  return (
    <div className="space-y-8">
      {/* Section Header */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Key className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">API Keys</h2>
        </div>

        {/* Deepgram API Key */}
        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5 space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-emerald-400 to-teal-500 text-sm font-bold text-white shadow-lg">
              DG
            </div>
            <div>
              <h3 className="font-semibold">Deepgram</h3>
              <p className="text-xs text-muted-foreground">Real-time speech recognition</p>
            </div>
          </div>
          <ApiKeyInput
            label="API Key"
            value={preferences.deepgram_api_key || ""}
            onChange={(value) => onUpdate("deepgram_api_key", value)}
            placeholder="Enter your Deepgram API key"
            helpText="Get your free API key at"
            helpLink={{ text: "console.deepgram.com", url: "https://console.deepgram.com" }}
            testEndpoint={testDeepgram}
          />
        </div>

        {/* Groq API Key */}
        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5 space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-orange-400 to-rose-500 text-sm font-bold text-white shadow-lg">
              GQ
            </div>
            <div>
              <h3 className="font-semibold">Groq</h3>
              <p className="text-xs text-muted-foreground">Fast Whisper inference</p>
            </div>
          </div>
          <ApiKeyInput
            label="API Key"
            value={preferences.groq_api_key || ""}
            onChange={(value) => onUpdate("groq_api_key", value)}
            placeholder="Enter your Groq API key"
            helpText="Get your API key at"
            helpLink={{ text: "console.groq.com", url: "https://console.groq.com" }}
          />
        </div>
      </section>

      {/* Security Info */}
      <section className="space-y-5">
        <div className="flex items-center gap-2">
          <Shield className="h-4 w-4 text-accent" />
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Security</h2>
        </div>

        <div className="rounded-2xl border border-white/[0.06] bg-white/[0.02] p-4 sm:p-5">
          <div className="flex items-start gap-3">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-white/[0.04]">
              <Shield className="h-4 w-4 text-emerald-400" />
            </div>
            <div className="space-y-2 text-sm text-muted-foreground">
              <p>
                Your API keys are stored securely in your system keychain and never leave your device
                except to authenticate with the respective services.
              </p>
              <p className="text-xs">
                Keys are encrypted at rest and transmitted only over HTTPS.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Alternative Options */}
      <section className="rounded-2xl border border-dashed border-white/[0.08] bg-white/[0.01] p-4 sm:p-5">
        <div className="flex items-start gap-3">
          <Sparkles className="h-5 w-5 text-accent shrink-0 mt-0.5" />
          <div>
            <h4 className="text-sm font-medium text-foreground mb-1">
              Don't want to manage API keys?
            </h4>
            <p className="text-xs text-muted-foreground leading-relaxed">
              Consider using <span className="text-foreground font-medium">Groq Whisper</span> (cloud) or{" "}
              <span className="text-foreground font-medium">Whisper Local</span> (on-device) for a simpler
              experience. Switch providers in the Transcription tab.
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}
