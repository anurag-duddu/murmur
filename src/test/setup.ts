import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    listen: vi.fn(() => Promise.resolve(() => {})),
    emit: vi.fn(),
  })),
  Window: {
    getByLabel: vi.fn(),
  },
}));

// Mock GSAP
vi.mock('gsap', () => ({
  gsap: {
    to: vi.fn(),
    from: vi.fn(),
    fromTo: vi.fn(),
    set: vi.fn(),
    timeline: vi.fn(() => ({
      to: vi.fn().mockReturnThis(),
      from: vi.fn().mockReturnThis(),
      add: vi.fn().mockReturnThis(),
      pause: vi.fn().mockReturnThis(),
      play: vi.fn().mockReturnThis(),
      kill: vi.fn(),
    })),
    registerPlugin: vi.fn(),
  },
  default: {
    to: vi.fn(),
    from: vi.fn(),
    fromTo: vi.fn(),
    set: vi.fn(),
    timeline: vi.fn(() => ({
      to: vi.fn().mockReturnThis(),
      from: vi.fn().mockReturnThis(),
      add: vi.fn().mockReturnThis(),
      pause: vi.fn().mockReturnThis(),
      play: vi.fn().mockReturnThis(),
      kill: vi.fn(),
    })),
    registerPlugin: vi.fn(),
  },
}));
