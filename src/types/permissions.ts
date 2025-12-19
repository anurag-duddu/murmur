export type PermissionState = "granted" | "denied" | "undetermined";

export interface PermissionStatus {
  microphone: PermissionState;
  accessibility: boolean;
}

export interface MicrophoneDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export const DEFAULT_PERMISSION_STATUS: PermissionStatus = {
  microphone: "undetermined",
  accessibility: false,
};
