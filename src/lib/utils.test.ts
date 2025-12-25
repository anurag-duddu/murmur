import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('cn utility', () => {
  describe('basic usage', () => {
    it('merges class names', () => {
      const result = cn('foo', 'bar');
      expect(result).toBe('foo bar');
    });

    it('handles single class name', () => {
      const result = cn('foo');
      expect(result).toBe('foo');
    });

    it('handles empty input', () => {
      const result = cn();
      expect(result).toBe('');
    });
  });

  describe('conditional classes', () => {
    it('includes truthy conditional classes', () => {
      const result = cn('foo', true && 'bar');
      expect(result).toBe('foo bar');
    });

    it('excludes falsy conditional classes', () => {
      const result = cn('foo', false && 'bar');
      expect(result).toBe('foo');
    });

    it('handles undefined values', () => {
      const result = cn('foo', undefined, 'bar');
      expect(result).toBe('foo bar');
    });

    it('handles null values', () => {
      const result = cn('foo', null, 'bar');
      expect(result).toBe('foo bar');
    });
  });

  describe('tailwind merge', () => {
    it('merges conflicting Tailwind classes', () => {
      const result = cn('p-2', 'p-4');
      expect(result).toBe('p-4');
    });

    it('merges conflicting padding classes', () => {
      const result = cn('px-2 py-1', 'p-3');
      expect(result).toBe('p-3');
    });

    it('merges conflicting color classes', () => {
      const result = cn('text-red-500', 'text-blue-500');
      expect(result).toBe('text-blue-500');
    });

    it('merges conflicting background classes', () => {
      const result = cn('bg-red-500', 'bg-blue-500');
      expect(result).toBe('bg-blue-500');
    });

    it('keeps non-conflicting classes', () => {
      const result = cn('p-2', 'm-4');
      expect(result).toBe('p-2 m-4');
    });

    it('merges width classes correctly', () => {
      const result = cn('w-full', 'w-1/2');
      expect(result).toBe('w-1/2');
    });

    it('merges flex classes correctly', () => {
      const result = cn('flex flex-row', 'flex-col');
      expect(result).toBe('flex flex-col');
    });
  });

  describe('array input', () => {
    it('handles array of class names', () => {
      const result = cn(['foo', 'bar']);
      expect(result).toBe('foo bar');
    });

    it('handles mixed array and string input', () => {
      const result = cn('foo', ['bar', 'baz']);
      expect(result).toBe('foo bar baz');
    });
  });

  describe('object input', () => {
    it('includes classes with truthy values', () => {
      const result = cn({ foo: true, bar: false, baz: true });
      expect(result).toBe('foo baz');
    });

    it('handles mixed object and string input', () => {
      const result = cn('base', { active: true, disabled: false });
      expect(result).toBe('base active');
    });
  });

  describe('real-world usage patterns', () => {
    it('handles button variant pattern', () => {
      const isActive = true;
      const isDisabled = false;
      const result = cn(
        'px-4 py-2 rounded',
        isActive && 'bg-blue-500 text-white',
        isDisabled && 'opacity-50 cursor-not-allowed'
      );
      expect(result).toBe('px-4 py-2 rounded bg-blue-500 text-white');
    });

    it('handles component prop override pattern', () => {
      const baseClasses = 'p-4 bg-white rounded-lg';
      const propClasses = 'p-6 bg-gray-100';
      const result = cn(baseClasses, propClasses);
      expect(result).toBe('rounded-lg p-6 bg-gray-100');
    });

    it('handles dark mode variant pattern', () => {
      const result = cn(
        'bg-white text-black',
        'dark:bg-black dark:text-white'
      );
      expect(result).toBe('bg-white text-black dark:bg-black dark:text-white');
    });
  });
});
