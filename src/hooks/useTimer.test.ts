import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useTimer } from './useTimer';

describe('useTimer', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('initialization', () => {
    it('initializes with 0 elapsed time by default', () => {
      const { result } = renderHook(() => useTimer());
      expect(result.current.elapsedMs).toBe(0);
      expect(result.current.isRunning).toBe(false);
    });

    it('initializes with provided duration', () => {
      const { result } = renderHook(() => useTimer(5000));
      expect(result.current.elapsedMs).toBe(5000);
    });

    it('formats time correctly at start', () => {
      const { result } = renderHook(() => useTimer());
      expect(result.current.formatted).toBe('0:00');
    });
  });

  describe('start/stop', () => {
    it('starts the timer', () => {
      const { result } = renderHook(() => useTimer());

      act(() => {
        result.current.start();
      });

      expect(result.current.isRunning).toBe(true);
    });

    it('stops the timer', () => {
      const { result } = renderHook(() => useTimer());

      act(() => {
        result.current.start();
      });

      act(() => {
        result.current.stop();
      });

      expect(result.current.isRunning).toBe(false);
    });
  });

  describe('reset', () => {
    it('resets timer to 0', () => {
      const { result } = renderHook(() => useTimer(10000));

      act(() => {
        result.current.start();
      });

      act(() => {
        result.current.reset();
      });

      expect(result.current.elapsedMs).toBe(0);
      expect(result.current.isRunning).toBe(false);
    });
  });

  describe('syncWithBackend', () => {
    it('syncs with backend duration', () => {
      const { result } = renderHook(() => useTimer());

      act(() => {
        result.current.syncWithBackend(3500);
      });

      // The sync should update the offset
      expect(result.current.elapsedMs).toBe(0); // Not running yet
    });
  });

  describe('time formatting', () => {
    it('formats 0 seconds correctly', () => {
      const { result } = renderHook(() => useTimer(0));
      expect(result.current.formatted).toBe('0:00');
    });

    it('formats 59 seconds correctly', () => {
      const { result } = renderHook(() => useTimer(59000));
      expect(result.current.formatted).toBe('0:59');
    });

    it('formats 1 minute correctly', () => {
      const { result } = renderHook(() => useTimer(60000));
      expect(result.current.formatted).toBe('1:00');
    });

    it('formats 1 minute 30 seconds correctly', () => {
      const { result } = renderHook(() => useTimer(90000));
      expect(result.current.formatted).toBe('1:30');
    });

    it('formats 10 minutes correctly', () => {
      const { result } = renderHook(() => useTimer(600000));
      expect(result.current.formatted).toBe('10:00');
    });

    it('pads single-digit seconds with leading zero', () => {
      const { result } = renderHook(() => useTimer(65000)); // 1:05
      expect(result.current.formatted).toBe('1:05');
    });
  });
});
