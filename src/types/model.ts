export interface ModelStatus {
  downloaded: boolean;
  downloading: boolean;
  progress: number;
  size_bytes: number;
  path: string | null;
}

export interface ModelDownloadProgress {
  progress: number;
  downloaded_bytes: number;
  total_bytes: number;
}

export const DEFAULT_MODEL_STATUS: ModelStatus = {
  downloaded: false,
  downloading: false,
  progress: 0,
  size_bytes: 0,
  path: null,
};

// Model info constants
export const MODEL_NAME = "large-v3-turbo";
export const MODEL_SIZE_MB = 547;
