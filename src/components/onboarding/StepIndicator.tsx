import { cn } from "@/lib/utils";

interface StepIndicatorProps {
  currentStep: number;
  totalSteps: number;
}

export function StepIndicator({ currentStep, totalSteps }: StepIndicatorProps) {
  return (
    <div className="flex justify-center gap-2">
      {Array.from({ length: totalSteps }).map((_, index) => {
        const stepNum = index + 1;
        const isActive = stepNum === currentStep;
        const isCompleted = stepNum < currentStep;

        return (
          <div
            key={index}
            className={cn(
              "h-2 rounded-full transition-all duration-300",
              isActive && "w-6 bg-primary",
              isCompleted && "w-2 bg-success",
              !isActive && !isCompleted && "w-2 bg-white/30"
            )}
          />
        );
      })}
    </div>
  );
}
