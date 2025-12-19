import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import { SPOKEN_LANGUAGES } from "@/types";

interface LanguageGridProps {
  selected: string[];
  onChange: (languages: string[]) => void;
  compact?: boolean;
}

export function LanguageGrid({ selected, onChange, compact = false }: LanguageGridProps) {
  const toggleLanguage = (code: string) => {
    if (selected.includes(code)) {
      // Don't allow deselecting if it's the last one
      if (selected.length > 1) {
        onChange(selected.filter((c) => c !== code));
      }
    } else {
      onChange([...selected, code]);
    }
  };

  return (
    <div className={cn("grid grid-cols-2 gap-2", compact && "gap-1.5")}>
      {SPOKEN_LANGUAGES.map((lang) => {
        const isSelected = selected.includes(lang.code);
        return (
          <button
            key={lang.code}
            type="button"
            onClick={() => toggleLanguage(lang.code)}
            className={cn(
              "flex items-center gap-2.5 rounded-lg border p-3 text-left transition-all duration-150",
              compact && "p-2.5",
              isSelected
                ? "border-primary bg-primary/5"
                : "border-border bg-card hover:border-primary/50"
            )}
          >
            <Checkbox
              checked={isSelected}
              className="pointer-events-none"
              tabIndex={-1}
            />
            <Label className={cn("cursor-pointer text-sm font-normal", compact && "text-xs")}>
              {lang.name}
            </Label>
          </button>
        );
      })}
    </div>
  );
}
