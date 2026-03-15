import { useEffect, useCallback, useRef, useState } from 'react';

interface ShortcutConfig {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  handler: () => void;
  description: string;
}

interface SequenceConfig {
  keys: [string, string];
  handler: () => void;
  description: string;
}

/**
 * Hook for registering single-key global keyboard shortcuts.
 * Ignores events when focus is inside an input, textarea, or contenteditable element.
 */
export function useKeyboardShortcuts(shortcuts: ShortcutConfig[]): void {
  const shortcutsRef = useRef(shortcuts);
  shortcutsRef.current = shortcuts;

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const target = e.target as HTMLElement;
      const isEditable =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable;

      for (const shortcut of shortcutsRef.current) {
        const metaMatch = shortcut.meta ? (e.metaKey || e.ctrlKey) : true;
        const ctrlMatch = shortcut.ctrl ? e.ctrlKey : true;
        const shiftMatch = shortcut.shift ? e.shiftKey : !e.shiftKey;

        // For shortcuts with meta/ctrl modifiers, allow even in editable fields
        const requiresMod = shortcut.meta || shortcut.ctrl;

        if (
          e.key.toLowerCase() === shortcut.key.toLowerCase() &&
          metaMatch &&
          ctrlMatch &&
          shiftMatch &&
          (!isEditable || requiresMod)
        ) {
          e.preventDefault();
          shortcut.handler();
          return;
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);
}

/**
 * Hook for two-key sequences like "G then I".
 * Returns the pending prefix key (e.g., "G") to display a visual indicator.
 */
export function useKeySequences(sequences: SequenceConfig[]): string | null {
  const [pendingKey, setPendingKey] = useState<string | null>(null);
  const sequencesRef = useRef(sequences);
  sequencesRef.current = sequences;
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearPending = useCallback(() => {
    setPendingKey(null);
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  useEffect(() => {
    let currentPending: string | null = null;

    function handleKeyDown(e: KeyboardEvent) {
      const target = e.target as HTMLElement;
      const isEditable =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable;

      if (isEditable) return;

      // Don't capture if any modifier is pressed (except shift for ? etc.)
      if (e.metaKey || e.ctrlKey || e.altKey) return;

      const key = e.key.toLowerCase();

      if (currentPending) {
        // We have a pending first key, check for second key match
        for (const seq of sequencesRef.current) {
          if (
            seq.keys[0].toLowerCase() === currentPending &&
            seq.keys[1].toLowerCase() === key
          ) {
            e.preventDefault();
            seq.handler();
            currentPending = null;
            clearPending();
            return;
          }
        }
        // No match, clear pending
        currentPending = null;
        clearPending();
        return;
      }

      // Check if this key is a valid first key in any sequence
      const isFirstKey = sequencesRef.current.some(
        (seq) => seq.keys[0].toLowerCase() === key
      );

      if (isFirstKey) {
        e.preventDefault();
        currentPending = key;
        setPendingKey(key);

        // Clear after 1 second if no second key
        if (timeoutRef.current) clearTimeout(timeoutRef.current);
        timeoutRef.current = setTimeout(() => {
          currentPending = null;
          setPendingKey(null);
        }, 1000);
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [clearPending]);

  return pendingKey;
}
