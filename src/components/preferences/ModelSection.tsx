import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useModelStatus } from "@/hooks";
import { cn } from "@/lib/utils";
import { Download, Trash2, Check, Loader2 } from "lucide-react";
import { MODEL_NAME, MODEL_SIZE_MB } from "@/types";

export function ModelSection() {
  const {
    isDownloaded,
    isDownloading,
    progress,
    status,
    downloadModel,
    deleteModel,
  } = useModelStatus();

  const sizeDisplay = status.size_bytes > 0
    ? `${Math.round(status.size_bytes / (1024 * 1024))} MB`
    : `${MODEL_SIZE_MB} MB`;

  return (
    <div className="rounded-xl bg-muted/50 p-4">
      <h3 className="mb-3 font-semibold">Whisper Model</h3>
      <div className="rounded-lg border border-border bg-card p-4">
        {/* Model Info */}
        <div className="flex items-center justify-between">
          <div>
            <p className="font-medium">{MODEL_NAME}</p>
            <p className="text-sm text-muted-foreground">{sizeDisplay}</p>
          </div>
          <div
            className={cn(
              "text-sm font-medium",
              isDownloaded
                ? "text-success"
                : isDownloading
                ? "text-warning"
                : "text-muted-foreground"
            )}
          >
            {isDownloaded
              ? "Downloaded and ready"
              : isDownloading
              ? "Downloading..."
              : "Not downloaded"}
          </div>
        </div>

        {/* Progress Bar */}
        {isDownloading && (
          <div className="mt-4 space-y-2">
            <Progress value={progress} />
            <p className="text-center text-sm text-muted-foreground">
              {Math.round(progress)}%
            </p>
          </div>
        )}

        {/* Actions */}
        <div className="mt-4 flex gap-2">
          {!isDownloaded ? (
            <Button
              onClick={downloadModel}
              disabled={isDownloading}
              className="flex-1"
            >
              {isDownloading ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Download className="mr-2 h-4 w-4" />
              )}
              {isDownloading ? "Downloading..." : "Download Model"}
            </Button>
          ) : (
            <>
              <div className="flex flex-1 items-center justify-center gap-2 text-sm text-success">
                <Check className="h-4 w-4" />
                Ready to use
              </div>
              <Button
                variant="outline"
                onClick={deleteModel}
                className="text-destructive hover:text-destructive"
              >
                <Trash2 className="mr-2 h-4 w-4" />
                Delete
              </Button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
