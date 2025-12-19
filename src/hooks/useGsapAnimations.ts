import { useRef, useEffect, useCallback } from "react";
import gsap from "gsap";

/**
 * GSAP animation utilities for smooth, professional animations
 */

// Animation presets for consistent timing
export const ANIMATION_PRESETS = {
  // Overlay entrance - bouncy scale up
  overlayEnter: {
    duration: 0.5,
    ease: "back.out(1.7)",
    scale: { from: 0.8, to: 1 },
    opacity: { from: 0, to: 1 },
    y: { from: 20, to: 0 },
  },
  // Overlay exit - smooth fade down
  overlayExit: {
    duration: 0.3,
    ease: "power2.in",
    scale: { from: 1, to: 0.95 },
    opacity: { from: 1, to: 0 },
    y: { from: 0, to: 10 },
  },
  // Tab content switch
  tabEnter: {
    duration: 0.4,
    ease: "power3.out",
    opacity: { from: 0, to: 1 },
    y: { from: 12, to: 0 },
  },
  // Card selection pulse
  cardSelect: {
    duration: 0.3,
    ease: "power2.out",
    scale: [1, 1.02, 1],
  },
  // Stagger reveal for lists
  staggerReveal: {
    duration: 0.4,
    ease: "power2.out",
    stagger: 0.08,
    opacity: { from: 0, to: 1 },
    y: { from: 15, to: 0 },
  },
};

/**
 * Hook for overlay pill entrance/exit animations
 */
export function useOverlayAnimation() {
  const containerRef = useRef<HTMLDivElement>(null);
  const timelineRef = useRef<gsap.core.Timeline | null>(null);

  const animateIn = useCallback(() => {
    if (!containerRef.current) return;

    // Kill any existing animation
    timelineRef.current?.kill();

    const { overlayEnter } = ANIMATION_PRESETS;

    timelineRef.current = gsap.timeline();

    timelineRef.current.fromTo(
      containerRef.current,
      {
        scale: overlayEnter.scale.from,
        opacity: overlayEnter.opacity.from,
        y: overlayEnter.y.from,
      },
      {
        scale: overlayEnter.scale.to,
        opacity: overlayEnter.opacity.to,
        y: overlayEnter.y.to,
        duration: overlayEnter.duration,
        ease: overlayEnter.ease,
      }
    );
  }, []);

  const animateOut = useCallback(() => {
    if (!containerRef.current) return Promise.resolve();

    const { overlayExit } = ANIMATION_PRESETS;

    return new Promise<void>((resolve) => {
      gsap.to(containerRef.current, {
        scale: overlayExit.scale.to,
        opacity: overlayExit.opacity.to,
        y: overlayExit.y.to,
        duration: overlayExit.duration,
        ease: overlayExit.ease,
        onComplete: resolve,
      });
    });
  }, []);

  // Animate in on mount
  useEffect(() => {
    animateIn();
    return () => {
      timelineRef.current?.kill();
    };
  }, [animateIn]);

  return { containerRef, animateIn, animateOut };
}

/**
 * Hook for recording dot pulse animation
 */
export function useRecordingPulse(isRecording: boolean) {
  const dotRef = useRef<HTMLDivElement>(null);
  const pulseRef = useRef<gsap.core.Tween | null>(null);

  useEffect(() => {
    if (!dotRef.current) return;

    // Kill existing pulse
    pulseRef.current?.kill();

    if (isRecording) {
      // Create organic pulse with GSAP
      pulseRef.current = gsap.to(dotRef.current, {
        scale: 1.2,
        boxShadow: "0 0 20px 6px rgba(255, 59, 48, 0.5)",
        duration: 0.8,
        repeat: -1,
        yoyo: true,
        ease: "sine.inOut",
      });
    } else {
      // Reset to normal
      gsap.to(dotRef.current, {
        scale: 1,
        boxShadow: "0 0 0 0 rgba(255, 59, 48, 0)",
        duration: 0.3,
        ease: "power2.out",
      });
    }

    return () => {
      pulseRef.current?.kill();
    };
  }, [isRecording]);

  return dotRef;
}

/**
 * Hook for tab content transitions
 */
export function useTabTransition(activeTab: string) {
  const contentRef = useRef<HTMLDivElement>(null);
  const prevTabRef = useRef(activeTab);

  useEffect(() => {
    if (!contentRef.current || prevTabRef.current === activeTab) return;

    const { tabEnter } = ANIMATION_PRESETS;

    gsap.fromTo(
      contentRef.current,
      {
        opacity: tabEnter.opacity.from,
        y: tabEnter.y.from,
      },
      {
        opacity: tabEnter.opacity.to,
        y: tabEnter.y.to,
        duration: tabEnter.duration,
        ease: tabEnter.ease,
      }
    );

    prevTabRef.current = activeTab;
  }, [activeTab]);

  return contentRef;
}

/**
 * Hook for card selection animation
 */
export function useCardSelectAnimation() {
  const cardRef = useRef<HTMLButtonElement>(null);

  const animateSelect = useCallback(() => {
    if (!cardRef.current) return;

    gsap.to(cardRef.current, {
      keyframes: {
        scale: ANIMATION_PRESETS.cardSelect.scale,
      },
      duration: ANIMATION_PRESETS.cardSelect.duration,
      ease: ANIMATION_PRESETS.cardSelect.ease,
    });
  }, []);

  return { cardRef, animateSelect };
}

/**
 * Hook for staggered list reveal
 */
export function useStaggerReveal<T extends HTMLElement>() {
  const containerRef = useRef<T>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const items = containerRef.current.children;
    if (items.length === 0) return;

    const { staggerReveal } = ANIMATION_PRESETS;

    gsap.fromTo(
      items,
      {
        opacity: staggerReveal.opacity.from,
        y: staggerReveal.y.from,
      },
      {
        opacity: staggerReveal.opacity.to,
        y: staggerReveal.y.to,
        duration: staggerReveal.duration,
        ease: staggerReveal.ease,
        stagger: staggerReveal.stagger,
      }
    );
  }, []);

  return containerRef;
}

/**
 * Hook for waveform bar animations - more organic than CSS
 */
export function useWaveformAnimation(
  audioLevel: number,
  barCount: number,
  barsRef: React.MutableRefObject<(HTMLSpanElement | null)[]>
) {
  const sensitivityRef = useRef<number[]>(
    Array.from({ length: barCount }, () => 0.5 + Math.random() * 1.0)
  );

  useEffect(() => {
    const NOISE_GATE = 0.1;
    const BASE_HEIGHT = 4;
    const MAX_HEIGHT = 28;

    const rawLevel = Math.max(0, Math.min(1, audioLevel));
    const adjustedLevel = rawLevel < NOISE_GATE
      ? 0
      : (rawLevel - NOISE_GATE) / (1.0 - NOISE_GATE);

    const center = barCount / 2;

    barsRef.current.forEach((bar, index) => {
      if (!bar) return;

      const distanceFromCenter = Math.abs(index - center) / center;
      const positionMultiplier = 1.0 - (distanceFromCenter * 0.4);
      const sensitivityAdjustedLevel = adjustedLevel * positionMultiplier * sensitivityRef.current[index];
      const targetHeight = BASE_HEIGHT + (sensitivityAdjustedLevel * (MAX_HEIGHT - BASE_HEIGHT));

      gsap.to(bar, {
        height: Math.max(BASE_HEIGHT, Math.min(MAX_HEIGHT, targetHeight)),
        duration: 0.08,
        ease: "power1.out",
      });
    });
  }, [audioLevel, barCount, barsRef]);
}

/**
 * Generic entrance animation hook
 */
export function useEntranceAnimation(options?: {
  delay?: number;
  duration?: number;
  y?: number;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ref.current) return;

    gsap.fromTo(
      ref.current,
      {
        opacity: 0,
        y: options?.y ?? 20,
      },
      {
        opacity: 1,
        y: 0,
        duration: options?.duration ?? 0.5,
        delay: options?.delay ?? 0,
        ease: "power3.out",
      }
    );
  }, [options?.delay, options?.duration, options?.y]);

  return ref;
}
