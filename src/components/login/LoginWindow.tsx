import { useState, useCallback, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { tauriCommands, tauriEvents } from "@/lib/tauri";
import { motion } from "framer-motion";
import type { AuthState } from "@/types/auth";

// Show "Having trouble?" after this many seconds
const AUTH_TIMEOUT_SECONDS = 30;

export function LoginWindow() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showTrouble, setShowTrouble] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Listen for auth state changes (callback from OAuth flow)
  useEffect(() => {
    const unlisten = tauriEvents.onAuthStateChanged((state: AuthState) => {
      if (state.is_authenticated) {
        // Auth successful - window will be closed by backend
        setIsLoading(false);
        setError(null);
        setShowTrouble(false);
        // Clear timeout
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
          timeoutRef.current = null;
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      // Cleanup timeout on unmount
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const handleLogin = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    setShowTrouble(false);

    // Clear any existing timeout
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    try {
      await tauriCommands.startAuth();
      // Browser will open, callback will be handled by backend
      // Keep loading state while user authenticates in browser

      // Start timeout - show "Having trouble?" after 30 seconds
      timeoutRef.current = setTimeout(() => {
        setShowTrouble(true);
      }, AUTH_TIMEOUT_SECONDS * 1000);
    } catch (err) {
      console.error("Failed to start auth:", err);
      setError("Failed to start sign in. Please try again.");
      setIsLoading(false);
    }
  }, []);

  const handleRetry = useCallback(() => {
    // Clear timeout and reset state, then try again
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setShowTrouble(false);
    setIsLoading(false);
    setError(null);
  }, []);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center glass-window p-10 relative overflow-hidden">
      {/* Ambient background glow */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-96 h-96 bg-accent/10 rounded-full blur-[100px] pointer-events-none" />
      <div className="absolute bottom-1/4 right-1/4 w-64 h-64 bg-blue-500/8 rounded-full blur-[80px] pointer-events-none" />

      <div className="relative w-full max-w-sm text-center">
        {/* Logo */}
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          className="mb-8"
        >
          <h1 className="text-4xl font-bold text-white">Keyhold</h1>
          <p className="mt-2 text-white/50">Voice to text, effortlessly</p>
        </motion.div>

        {/* Login Card */}
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 0.1 }}
          className="rounded-xl bg-white/5 border border-white/10 p-6 backdrop-blur-sm"
        >
          <h2 className="text-lg font-medium text-white mb-2">Welcome</h2>
          <p className="text-sm text-white/60 mb-6">
            Sign in to start using Keyhold
          </p>

          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mb-4 p-3 rounded-lg bg-red-500/20 border border-red-500/30"
            >
              <p className="text-sm text-red-200">{error}</p>
            </motion.div>
          )}

          <Button
            onClick={handleLogin}
            disabled={isLoading}
            className="w-full"
            size="lg"
          >
            {isLoading ? (
              <span className="flex items-center gap-2">
                <svg
                  className="animate-spin h-4 w-4"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                Opening browser...
              </span>
            ) : (
              "Sign in"
            )}
          </Button>

          {isLoading && !showTrouble && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="mt-4 space-y-2"
            >
              <p className="text-xs text-white/40">
                Complete sign in in your browser, then return here
              </p>
              <button
                onClick={handleRetry}
                className="text-xs text-white/50 hover:text-white/70 underline underline-offset-2"
              >
                Cancel and try again
              </button>
            </motion.div>
          )}

          {/* Show "Having trouble?" after timeout */}
          {isLoading && showTrouble && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mt-4 space-y-3"
            >
              <div className="rounded-lg bg-amber-500/10 border border-amber-500/20 p-3">
                <p className="text-sm font-medium text-amber-200 mb-1">
                  Having trouble?
                </p>
                <p className="text-xs text-amber-200/70">
                  If the browser didn't open or you closed it accidentally, try again.
                </p>
              </div>
              <div className="flex gap-2">
                <Button
                  onClick={handleLogin}
                  variant="outline"
                  className="flex-1 bg-white/[0.05] text-white hover:bg-white/[0.08] border-white/[0.1]"
                >
                  Try Again
                </Button>
                <Button
                  onClick={handleRetry}
                  variant="outline"
                  className="bg-white/[0.05] text-white/60 hover:bg-white/[0.08] border-white/[0.1]"
                >
                  Cancel
                </Button>
              </div>
            </motion.div>
          )}
        </motion.div>

        <p className="mt-6 text-xs text-white/40">
          Powered by WorkOS
        </p>
      </div>
    </div>
  );
}
