import { useState, useCallback, useEffect, useRef } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GeneralTab } from "./GeneralTab";
import { AudioTab } from "./AudioTab";
import { StickyActionBar } from "./StickyActionBar";
import { usePreferences, useEntranceAnimation } from "@/hooks";
import { Settings, Volume2, Sparkles } from "lucide-react";
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
      <div className="flex h-screen items-center justify-center glass-window">
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
    <div className="flex h-screen flex-col glass-window overflow-hidden">
      {/* Header with glass panel effect */}
      <header ref={headerRef} className="relative glass-panel border-x-0 border-t-0 rounded-none">
        {/* Ambient accent glow */}
        <div className="absolute inset-0 bg-gradient-to-r from-accent/8 via-transparent to-accent/8 pointer-events-none" />
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-32 h-px bg-gradient-to-r from-transparent via-accent/40 to-transparent" />

        <div className="relative flex items-center justify-between px-4 sm:px-6 py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-gradient-to-br from-accent to-orange-600 shadow-lg shadow-accent/25">
              <Sparkles className="h-4 w-4 text-white" />
            </div>
            <div>
              <h1 className="text-lg font-semibold text-white">Keyhold</h1>
              <p className="text-xs text-white/60 -mt-0.5">Preferences</p>
            </div>
          </div>
          <div className="hidden sm:flex items-center gap-2 text-xs text-white/50">
            <kbd className="rounded-md bg-white/[0.08] border border-white/[0.12] px-2 py-1 text-[10px] font-medium text-white/70">⌘S</kbd>
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
          {/* Tab navigation - glass style */}
          <div className="border-b border-white/[0.08] px-2 sm:px-4 py-2 overflow-x-auto scrollbar-hide bg-white/[0.02]">
            <TabsList className="inline-flex h-10 w-full min-w-max gap-1 bg-transparent p-0">
              <TabsTrigger
                value="general"
                className="flex-1 sm:flex-none gap-2 rounded-lg px-3 sm:px-4 py-2 text-sm font-medium text-white/50 transition-all data-[state=active]:bg-white/[0.08] data-[state=active]:text-white data-[state=active]:shadow-sm data-[state=active]:border data-[state=active]:border-white/[0.1] hover:text-white/80 hover:bg-white/[0.04]"
              >
                <Settings className="h-4 w-4" />
                <span className="hidden xs:inline">General</span>
              </TabsTrigger>
              <TabsTrigger
                value="audio"
                className="flex-1 sm:flex-none gap-2 rounded-lg px-3 sm:px-4 py-2 text-sm font-medium text-white/50 transition-all data-[state=active]:bg-white/[0.08] data-[state=active]:text-white data-[state=active]:shadow-sm data-[state=active]:border data-[state=active]:border-white/[0.1] hover:text-white/80 hover:bg-white/[0.04]"
              >
                <Volume2 className="h-4 w-4" />
                <span className="hidden xs:inline">Audio</span>
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

              <TabsContent value="audio" className="mt-0">
                <AudioTab
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
