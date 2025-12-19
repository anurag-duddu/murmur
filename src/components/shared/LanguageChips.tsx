import { Badge } from "@/components/ui/badge";
import { LANGUAGE_NAMES } from "@/types";

interface LanguageChipsProps {
  languages: string[];
  onEdit?: () => void;
}

export function LanguageChips({ languages }: LanguageChipsProps) {
  return (
    <div className="flex flex-wrap gap-1.5">
      {languages.map((code) => (
        <Badge key={code} variant="outline" className="font-normal">
          {LANGUAGE_NAMES[code] || code}
        </Badge>
      ))}
    </div>
  );
}
