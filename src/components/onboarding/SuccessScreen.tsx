import { Button } from "@/components/ui/button";
import { Check } from "lucide-react";
import { motion } from "framer-motion";

interface SuccessScreenProps {
  hotkey: string;
  onFinish: () => void;
}

export function SuccessScreen({ hotkey, onFinish }: SuccessScreenProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className="text-center"
    >
      {/* Success Icon with glow */}
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: 0.2, type: "spring", stiffness: 200 }}
        className="relative mx-auto mb-6 h-20 w-20"
      >
        <div className="absolute inset-0 bg-success/30 rounded-full blur-xl scale-150" />
        <div className="relative flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-success to-emerald-600 shadow-lg shadow-success/30">
          <Check className="h-10 w-10 text-white" strokeWidth={3} />
        </div>
      </motion.div>

      {/* Success Text */}
      <h2 className="mb-3 text-2xl font-bold text-white">You're all set!</h2>
      <p className="mb-8 text-white/50 leading-relaxed">
        Press{" "}
        <span className="inline-flex items-center gap-1 rounded-md bg-white/[0.1] border border-white/[0.15] px-2.5 py-1 font-mono text-sm text-white/90">
          {hotkey}
        </span>{" "}
        anywhere to start dictating. Keyhold will transcribe your speech and
        paste it directly into any text field.
      </p>

      {/* Finish Button */}
      <Button
        onClick={onFinish}
        className="w-full bg-gradient-to-r from-success to-emerald-600 hover:from-success/90 hover:to-emerald-600/90 shadow-lg shadow-success/20"
        size="lg"
      >
        Start Using Keyhold
      </Button>
    </motion.div>
  );
}
