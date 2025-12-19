import { useState, useCallback, useEffect, useRef } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GeneralTab } from "./GeneralTab";
import { TranscriptionTab } from "./TranscriptionTab";
import { AudioTab } from "./AudioTab";
import { ApiTab } from "./ApiTab";
import { StickyActionBar } from "./StickyActionBar";
import { usePreferences, useEntranceAnimation } from "@/hooks";
import { Settings, Mic, Key, Volume2, Sparkles } from "lucide-react";
import gsap from "gsap";

export function PreferencesWindow() {
  const {
    preferences,
    isLoading,
    isSaving,
    hasChanges,
    updatePreference,
    savePreferences,
    resetPreferences,
  } = usePreferences();

  const [activeTab, setActiveTab] = useState("general");
  const [saveSuccess, setSaveSuccess] = useState(false);
  const prevTabRef = useRef(activeTab);
  const contentRef = useRef<HTMLDivElement>(null);

  // Header entrance animation
  const headerRef = useEntranceAnimation({ delay: 0, duration: 0.5, y: -20 });

  // Animate tab content on change
  useEffect(() => {
    if (prevTabRef.current !== activeTab && contentRef.current) {
      gsap.fromTo(
        contentRef.current,
        { opacity: 0, y: 12 },
        { opacity: 1, y: 0, duration: 0.35, ease: "power3.out" }
      );
      prevTabRef.current = activeTab;
    }
  }, [activeTab]);

  // Handle save with success feedback
  const handleSave = useCallback(async () => {
    const success = await savePreferences();
    if (success) {
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    }
  }, [savePreferences]);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        if (hasChanges) {
          handleSave();
        }
      }
      if (e.key === "Escape" && hasChanges) {
        resetPreferences();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [hasChanges, handleSave, resetPreferences]);

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <div className="relative">
            <div className="h-10 w-10 animate-spin rounded-full border-2 border-accent/30 border-t-accent" />
            <Sparkles className="absolute inset-0 m-auto h-4 w-4 text-accent animate-pulse" />
          </div>
          <p className="text-sm text-muted-foreground">Loading preferences...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col bg-background overflow-hidden">
      {/* Header with gradient accent */}
      <header ref={headerRef} className="relative border-b border-border/50 bg-gradient-to-b from-card to-background">
        {/* Ambient glow */}
        <div className="absolute inset-0 bg-gradient-to-r from-accent/5 via-transparent to-accent/5 pointer-events-none" />

        <div className="relative flex items-center justify-between px-4 sm:px-6 py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-accent to-orange-600 shadow-lg shadow-accent/20">
              <Sparkles className="h-4 w-4 text-white" />
            </div>
            <div>
              <h1 className="text-lg font-semibold">Murmur</h1>
              <p className="text-xs text-muted-foreground -mt-0.5">Preferences</p>
            </div>
          </div>
          <div className="hidden sm:flex items-center gap-2 text-xs text-muted-foreground">
            <kbd className="rounded-md bg-white/[0.06] border border-white/[0.08] px-2 py-1 text-[10px] font-medium">⌘S</kbd>
            <span>to save</span>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className="flex h-full flex-col"
        >
          {/* Tab navigation - responsive */}
          <div className="border-b border-border/50 px-2 sm:px-4 py-2 overflow-x-auto scrollbar-hide">
            <TabsList className="inline-flex h-10 w-full min-w-max gap-1 bg-transparent p-0">
              <TabsTrigger
                value="general"
                className="flex-1 sm:flex-none gap-2 rounded-lg px-3 sm:px-4 py-2 text-sm font-medium text-muted-foreground transition-all data-[state=active]:bg-white/[0.06] data-[state=active]:text-foreground data-[state=active]:shadow-sm hover:text-foreground hover:bg-white/[0.03]"
              >
                <Settings className="h-4 w-4" />
                <span className="hidden xs:inline">General</span>
              </TabsTrigger>
              <TabsTrigger
                value="transcription"
                className="flex-1 sm:flex-none gap-2 rounded-lg px-3 sm:px-4 py-2 text-sm font-medium text-muted-foreground transition-all data-[state=active]:bg-white/[0.06] data-[state=active]:text-foreground data-[state=active]:shadow-sm hover:text-foreground hover:bg-white/[0.03]"
              >
                <Mic className="h-4 w-4" />
                <span className="hidden xs:inline">Transcription</span>
              </TabsTrigger>
              <TabsTrigger
                value="audio"
                className="flex-1 sm:flex-none gap-2 rounded-lg px-3 sm:px-4 py-2 text-sm font-medium text-muted-foreground transition-all data-[state=active]:bg-white/[0.06] data-[state=active]:text-foreground data-[state=active]:shadow-sm hover:text-foreground hover:bg-white/[0.03]"
              >
                <Volume2 className="h-4 w-4" />
                <span className="hidden xs:inline">Audio</span>
              </TabsTrigger>
              <TabsTrigger
                value="api"
                className="flex-1 sm:flex-none gap-2 rounded-lg px-3 sm:px-4 py-2 text-sm font-medium text-muted-foreground transition-all data-[state=active]:bg-white/[0.06] data-[state=active]:text-foreground data-[state=active]:shadow-sm hover:text-foreground hover:bg-white/[0.03]"
              >
                <Key className="h-4 w-4" />
                <span className="hidden xs:inline">API Keys</span>
              </TabsTrigger>
            </TabsList>
          </div>

          {/* Tab content - responsive padding */}
          <div className="flex-1 overflow-y-auto">
            <div ref={contentRef} className="w-full max-w-2xl mx-auto px-4 sm:px-6 py-6">
              <TabsContent value="general" className="mt-0">
                <GeneralTab
                  preferences={preferences}
                  onUpdate={updatePreference}
                />
              </TabsContent>

              <TabsContent value="transcription" className="mt-0">
                <TranscriptionTab
                  preferences={preferences}
                  onUpdate={updatePreference}
                />
              </TabsContent>

              <TabsContent value="audio" className="mt-0">
                <AudioTab
                  preferences={preferences}
                  onUpdate={updatePreference}
                />
              </TabsContent>

              <TabsContent value="api" className="mt-0">
                <ApiTab
                  preferences={preferences}
                  onUpdate={updatePreference}
                />
              </TabsContent>
            </div>
          </div>
        </Tabs>
      </div>

      {/* Sticky Action Bar */}
      <StickyActionBar
        hasChanges={hasChanges}
        isSaving={isSaving}
        saveSuccess={saveSuccess}
        onSave={handleSave}
        onCancel={resetPreferences}
      />
    </div>
  );
}
