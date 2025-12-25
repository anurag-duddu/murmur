import { describe, it, expect } from 'vitest';
import {
  canStartRecording,
  canStopRecording,
  isProcessing,
  type RecordingState,
} from './recording';

describe('Recording State Helpers', () => {
  describe('canStartRecording', () => {
    it('returns true when state is idle', () => {
      expect(canStartRecording('idle')).toBe(true);
    });

    it('returns true when state is error', () => {
      expect(canStartRecording('error')).toBe(true);
    });

    it('returns false when state is recording', () => {
      expect(canStartRecording('recording')).toBe(false);
    });

    it('returns false when state is transcribing', () => {
      expect(canStartRecording('transcribing')).toBe(false);
    });

    it('returns false when state is enhancing', () => {
      expect(canStartRecording('enhancing')).toBe(false);
    });

    it('returns false when state is transforming', () => {
      expect(canStartRecording('transforming')).toBe(false);
    });
  });

  describe('canStopRecording', () => {
    it('returns true when state is recording', () => {
      expect(canStopRecording('recording')).toBe(true);
    });

    it('returns false when state is idle', () => {
      expect(canStopRecording('idle')).toBe(false);
    });

    it('returns false when state is transcribing', () => {
      expect(canStopRecording('transcribing')).toBe(false);
    });

    it('returns false when state is enhancing', () => {
      expect(canStopRecording('enhancing')).toBe(false);
    });

    it('returns false when state is error', () => {
      expect(canStopRecording('error')).toBe(false);
    });
  });

  describe('isProcessing', () => {
    it('returns true when state is transcribing', () => {
      expect(isProcessing('transcribing')).toBe(true);
    });

    it('returns true when state is enhancing', () => {
      expect(isProcessing('enhancing')).toBe(true);
    });

    it('returns true when state is transforming', () => {
      expect(isProcessing('transforming')).toBe(true);
    });

    it('returns false when state is idle', () => {
      expect(isProcessing('idle')).toBe(false);
    });

    it('returns false when state is recording', () => {
      expect(isProcessing('recording')).toBe(false);
    });

    it('returns false when state is error', () => {
      expect(isProcessing('error')).toBe(false);
    });
  });

  describe('RecordingState type coverage', () => {
    it('covers all valid states', () => {
      const allStates: RecordingState[] = [
        'idle',
        'recording',
        'transcribing',
        'enhancing',
        'transforming',
        'error',
      ];

      // Test each state against each helper
      allStates.forEach((state) => {
        expect(typeof canStartRecording(state)).toBe('boolean');
        expect(typeof canStopRecording(state)).toBe('boolean');
        expect(typeof isProcessing(state)).toBe('boolean');
      });
    });
  });
});
