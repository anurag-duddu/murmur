import { useState, useEffect, useCallback } from "react";
import { tauriCommands, tauriEvents } from "@/lib/tauri";
import type { ModelStatus } from "@/types";
import { DEFAULT_MODEL_STATUS } from "@/types";

export function useModelStatus() {
  const [status, setStatus] = useState<ModelStatus>(DEFAULT_MODEL_STATUS);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStatus = useCallback(async () => {
    try {
      const result = await tauriCommands.getModelStatus();
      setStatus(result);
      setError(null);
    } catch (err) {
      console.error("Failed to load model status:", err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  // Subscribe to download events
  useEffect(() => {
    const unsubProgress = tauriEvents.onModelProgress((progress) => {
      setStatus((prev) => ({
        ...prev,
        downloading: true,
        progress: progress.progress,
        size_bytes: progress.total_bytes,
      }));
    });

    const unsubComplete = tauriEvents.onModelComplete((path) => {
      setStatus((prev) => ({
        ...prev,
        downloading: false,
        downloaded: true,
        progress: 100,
        path,
      }));
    });

    const unsubError = tauriEvents.onModelError((err) => {
      setStatus((prev) => ({
        ...prev,
        downloading: false,
        progress: 0,
      }));
      setError(err);
    });

    return () => {
      unsubProgress.then((fn) => fn());
      unsubComplete.then((fn) => fn());
      unsubError.then((fn) => fn());
    };
  }, []);

  const downloadModel = useCallback(async () => {
    setError(null);
    try {
      await tauriCommands.downloadModel();
    } catch (err) {
      console.error("Failed to download model:", err);
      setError(String(err));
    }
  }, []);

  const deleteModel = useCallback(async () => {
    try {
      await tauriCommands.deleteModel();
      await loadStatus();
    } catch (err) {
      console.error("Failed to delete model:", err);
      setError(String(err));
    }
  }, [loadStatus]);

  return {
    status,
    isLoading,
    error,
    isDownloaded: status.downloaded,
    isDownloading: status.downloading,
    progress: status.progress,
    downloadModel,
    deleteModel,
    refresh: loadStatus,
  };
}
