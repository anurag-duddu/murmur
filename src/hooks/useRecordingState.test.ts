import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useRecordingState } from './useRecordingState';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Mock the Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('useRecordingState', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default mock for getOverlayState
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_overlay_state') {
        return { state: 'idle', message: null, recording_duration_ms: null };
      }
      return undefined;
    });
    // Default mock for listen
    vi.mocked(listen).mockResolvedValue(() => {});
  });

  describe('initialization', () => {
    it('initializes with idle state', async () => {
      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isIdle).toBe(true);
      });
    });

    it('fetches initial state from backend', async () => {
      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_overlay_state');
      });
    });

    it('sets up state change listener', async () => {
      renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(listen).toHaveBeenCalled();
      });
    });
  });

  describe('state properties', () => {
    it('isRecording is true when state is recording', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        state: 'recording',
        message: null,
        recording_duration_ms: 1000,
      });

      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isRecording).toBe(true);
      });
    });

    it('isProcessing is true when state is transcribing', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        state: 'transcribing',
        message: 'Processing...',
        recording_duration_ms: 5000,
      });

      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isProcessing).toBe(true);
      });
    });

    it('isProcessing is true when state is enhancing', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        state: 'enhancing',
        message: 'Enhancing...',
        recording_duration_ms: 5000,
      });

      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isProcessing).toBe(true);
      });
    });

    it('isError is true when state is error', async () => {
      vi.mocked(invoke).mockResolvedValueOnce({
        state: 'error',
        message: 'Something went wrong',
        recording_duration_ms: null,
      });

      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });
    });
  });

  describe('actions', () => {
    it('startRecording calls the backend', async () => {
      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isIdle).toBe(true);
      });

      await act(async () => {
        await result.current.startRecording();
      });

      expect(invoke).toHaveBeenCalledWith('start_recording');
    });

    it('stopRecording calls the backend', async () => {
      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isIdle).toBe(true);
      });

      await act(async () => {
        await result.current.stopRecording();
      });

      expect(invoke).toHaveBeenCalledWith('stop_recording');
    });

    it('cancelRecording calls the backend', async () => {
      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isIdle).toBe(true);
      });

      await act(async () => {
        await result.current.cancelRecording();
      });

      expect(invoke).toHaveBeenCalledWith('cancel_recording');
    });

    it('toggleRecording calls the backend', async () => {
      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isIdle).toBe(true);
      });

      await act(async () => {
        await result.current.toggleRecording();
      });

      expect(invoke).toHaveBeenCalledWith('toggle_recording');
    });
  });

  describe('error handling', () => {
    it('handles getOverlayState errors gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalled();
      });

      consoleSpy.mockRestore();
    });

    it('handles startRecording errors gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'start_recording') {
          throw new Error('Failed to start');
        }
        return { state: 'idle', message: null };
      });

      const { result } = renderHook(() => useRecordingState());

      await waitFor(() => {
        expect(result.current.isIdle).toBe(true);
      });

      await act(async () => {
        await result.current.startRecording();
      });

      expect(consoleSpy).toHaveBeenCalledWith('Failed to start recording:', expect.any(Error));
      consoleSpy.mockRestore();
    });
  });
});
