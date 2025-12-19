import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useLicenseInfo } from "@/hooks";
import { cn } from "@/lib/utils";
import { Check, X, Loader2 } from "lucide-react";

interface LicenseSectionProps {
  licenseKey: string;
  onLicenseChange: (key: string) => void;
}

export function LicenseSection({ licenseKey, onLicenseChange }: LicenseSectionProps) {
  const [inputKey, setInputKey] = useState(licenseKey);
  const { license, isActivating, error, activateLicense } = useLicenseInfo();

  const handleActivate = async () => {
    if (!inputKey.trim()) return;
    const result = await activateLicense(inputKey);
    if (result?.valid) {
      onLicenseChange(inputKey);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleActivate();
    }
  };

  return (
    <div className="rounded-xl bg-muted/50 p-4">
      <h3 className="mb-3 font-semibold">License Key</h3>
      <div className="space-y-3">
        <div className="flex gap-2">
          <Input
            type="text"
            placeholder="Enter your license key"
            value={inputKey}
            onChange={(e) => setInputKey(e.target.value)}
            onKeyDown={handleKeyDown}
            className="flex-1"
          />
          <Button
            onClick={handleActivate}
            disabled={isActivating || !inputKey.trim()}
          >
            {isActivating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Activate
          </Button>
        </div>

        {/* Status Message */}
        {(license.valid || error) && (
          <div
            className={cn(
              "flex items-center gap-2 rounded-lg px-3 py-2 text-sm",
              license.valid
                ? "bg-success/10 text-success"
                : "bg-destructive/10 text-destructive"
            )}
          >
            {license.valid ? (
              <>
                <Check className="h-4 w-4" />
                <span>
                  License activated:{" "}
                  {license.tier === "subscription" ? "Subscription" : "Lifetime"}
                </span>
              </>
            ) : (
              <>
                <X className="h-4 w-4" />
                <span>{error || "Invalid license key"}</span>
              </>
            )}
          </div>
        )}

        <p className="text-xs text-muted-foreground">
          Get your license at{" "}
          <a
            href="https://murmur.app"
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary hover:underline"
          >
            murmur.app
          </a>
        </p>
      </div>
    </div>
  );
}
