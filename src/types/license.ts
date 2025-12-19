export type LicenseTier = "free" | "subscription" | "lifetime";

export interface LicenseInfo {
  tier: LicenseTier;
  valid: boolean;
  license_key: string | null;
  expires_at: string | null;
}

export const DEFAULT_LICENSE_INFO: LicenseInfo = {
  tier: "free",
  valid: false,
  license_key: null,
  expires_at: null,
};

// License tier display names
export const LICENSE_TIER_NAMES: Record<LicenseTier, string> = {
  free: "Free",
  subscription: "Subscription",
  lifetime: "Lifetime",
};
