import { ProviderCard } from "./ProviderCard";
import { LicenseSection } from "./LicenseSection";
import { ModelSection } from "./ModelSection";
import type { Preferences, TranscriptionProvider } from "@/types";

interface TranscriptionTabProps {
  preferences: Preferences;
  onUpdate: <K extends keyof Preferences>(key: K, value: Preferences[K]) => void;
}

const PROVIDERS = [
  {
    id: "deepgram" as TranscriptionProvider,
    title: "Deepgram",
    subtitle: "Bring Your Own Key",
    description: "Fast cloud transcription with your API key",
    icon: "DG",
    iconColor: "from-emerald-400 to-teal-500",
    tags: [
      { label: "Free tier", variant: "success" as const },
      { label: "Real-time", variant: "secondary" as const },
    ],
  },
  {
    id: "whisperapi" as TranscriptionProvider,
    title: "Groq Whisper",
    subtitle: "Cloud API",
    description: "OpenAI Whisper powered by Groq's fast inference",
    icon: "GQ",
    iconColor: "from-orange-400 to-rose-500",
    tags: [
      { label: "Highest accuracy", variant: "default" as const },
      { label: "Multilingual", variant: "secondary" as const },
    ],
  },
  {
    id: "whisperlocal" as TranscriptionProvider,
    title: "Whisper Local",
    subtitle: "On-Device",
    description: "Run Whisper locally on your Mac",
    icon: "WL",
    iconColor: "from-violet-400 to-purple-500",
    tags: [
      { label: "Offline", variant: "default" as const },
      { label: "Private", variant: "secondary" as const },
    ],
  },
];

export function TranscriptionTab({ preferences, onUpdate }: TranscriptionTabProps) {
  const currentProvider = preferences.transcription_provider || "deepgram";
  const showLicense = currentProvider === "whisperapi" || currentProvider === "whisperlocal";
  const showModel = currentProvider === "whisperlocal";

  return (
    <div className="space-y-6">
      {/* Provider Selection */}
      <div className="space-y-3">
        {PROVIDERS.map((provider) => (
          <ProviderCard
            key={provider.id}
            provider={provider.id}
            title={provider.title}
            subtitle={provider.subtitle}
            description={provider.description}
            icon={provider.icon}
            iconColor={provider.iconColor}
            tags={provider.tags}
            selected={currentProvider === provider.id}
            onSelect={() => onUpdate("transcription_provider", provider.id)}
          />
        ))}
      </div>

      {/* License Section */}
      {showLicense && (
        <LicenseSection
          licenseKey={preferences.license_key || ""}
          onLicenseChange={(key) => onUpdate("license_key", key)}
        />
      )}

      {/* Model Section */}
      {showModel && <ModelSection />}
    </div>
  );
}
