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
      {/* Success Icon */}
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: 0.2, type: "spring", stiffness: 200 }}
        className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-success to-emerald-600"
      >
        <Check className="h-10 w-10 text-white" strokeWidth={3} />
      </motion.div>

      {/* Success Text */}
      <h2 className="mb-3 text-2xl font-bold text-white">You're all set!</h2>
      <p className="mb-8 text-white/60 leading-relaxed">
        Press{" "}
        <span className="inline-flex items-center gap-1 rounded-md bg-white/15 px-2.5 py-1 font-mono text-sm text-white">
          {hotkey}
        </span>{" "}
        anywhere to start dictating. Murmur will transcribe your speech and
        paste it directly into any text field.
      </p>

      {/* Finish Button */}
      <Button
        onClick={onFinish}
        className="w-full bg-gradient-to-r from-success to-emerald-600 hover:from-success/90 hover:to-emerald-600/90"
        size="lg"
      >
        Start Using Murmur
      </Button>
    </motion.div>
  );
}
