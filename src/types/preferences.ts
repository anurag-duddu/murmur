// Backend uses snake_case - must match exactly
export interface Preferences {
  recording_mode: "push-to-talk" | "toggle";
  hotkey: string;
  show_indicator: boolean;
  play_sounds: boolean;
  microphone: string;
  language: string;
  deepgram_api_key: string;
  groq_api_key: string;
  anthropic_api_key: string;
  transcription_provider: TranscriptionProvider;
  license_key: string | null;
  onboarding_complete: boolean;
  spoken_languages: string[];
}

export type TranscriptionProvider = "deepgram" | "whisperapi" | "whisperlocal";

export const DEFAULT_PREFERENCES: Preferences = {
  recording_mode: "toggle",
  hotkey: "Option+Space",
  show_indicator: true,
  play_sounds: true,
  microphone: "default",
  language: "en-US",
  deepgram_api_key: "",
  groq_api_key: "",
  anthropic_api_key: "",
  transcription_provider: "deepgram",
  license_key: null,
  onboarding_complete: false,
  spoken_languages: ["en"],
};

// Language display names
export const LANGUAGE_NAMES: Record<string, string> = {
  en: "English",
  hi: "Hindi",
  te: "Telugu",
  ta: "Tamil",
  es: "Spanish",
  fr: "French",
  de: "German",
  ja: "Japanese",
  zh: "Chinese",
  ko: "Korean",
  kn: "Kannada",
  ml: "Malayalam",
  bn: "Bengali",
  mr: "Marathi",
  gu: "Gujarati",
  pa: "Punjabi",
};

// Available spoken languages for selection
export const SPOKEN_LANGUAGES = [
  { code: "en", name: "English" },
  { code: "hi", name: "Hindi" },
  { code: "te", name: "Telugu" },
  { code: "ta", name: "Tamil" },
  { code: "es", name: "Spanish" },
  { code: "fr", name: "French" },
  { code: "de", name: "German" },
  { code: "ja", name: "Japanese" },
  { code: "zh", name: "Chinese" },
  { code: "ko", name: "Korean" },
];

// Transcription language options
export const TRANSCRIPTION_LANGUAGES = [
  { value: "en-US", label: "English (US)" },
  { value: "mixed", label: "Mixed (your languages)" },
  { value: "en-GB", label: "English (UK)" },
  { value: "en-AU", label: "English (Australia)" },
  { value: "en-IN", label: "English (India)" },
  // Indian Languages
  { value: "hi", label: "Hindi (हिन्दी)" },
  { value: "te", label: "Telugu (తెలుగు)" },
  { value: "ta", label: "Tamil (தமிழ்)" },
  { value: "kn", label: "Kannada (ಕನ್ನಡ)" },
  { value: "ml", label: "Malayalam (മലയാളം)" },
  { value: "bn", label: "Bengali (বাংলা)" },
  { value: "mr", label: "Marathi (मराठी)" },
  { value: "gu", label: "Gujarati (ગુજરાતી)" },
  { value: "pa", label: "Punjabi (ਪੰਜਾਬੀ)" },
  // European Languages
  { value: "es", label: "Spanish" },
  { value: "es-419", label: "Spanish (Latin America)" },
  { value: "fr", label: "French" },
  { value: "fr-CA", label: "French (Canada)" },
  { value: "de", label: "German" },
  { value: "it", label: "Italian" },
  { value: "pt", label: "Portuguese" },
  { value: "pt-BR", label: "Portuguese (Brazil)" },
  { value: "nl", label: "Dutch" },
  { value: "ja", label: "Japanese" },
  { value: "ko", label: "Korean" },
  { value: "zh-CN", label: "Chinese (Simplified)" },
  { value: "zh-TW", label: "Chinese (Traditional)" },
  { value: "ru", label: "Russian" },
  { value: "pl", label: "Polish" },
  { value: "tr", label: "Turkish" },
  { value: "uk", label: "Ukrainian" },
  { value: "vi", label: "Vietnamese" },
  { value: "id", label: "Indonesian" },
  { value: "th", label: "Thai" },
  { value: "sv", label: "Swedish" },
  { value: "da", label: "Danish" },
  { value: "no", label: "Norwegian" },
  { value: "fi", label: "Finnish" },
];

// Hotkey options
export const HOTKEY_OPTIONS = [
  { value: "Option+Space", label: "Option + Space (Default)" },
  { value: "CmdOrCtrl+Shift+D", label: "Cmd + Shift + D" },
  { value: "CmdOrCtrl+Option+Space", label: "Cmd + Option + Space" },
];
