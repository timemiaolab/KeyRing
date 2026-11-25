import { useEffect, useRef, useState } from 'react';
import { ModifierKey } from '../types/keyring';

export type StateMachinePhase =
  | 'idle'
  | 'holding'
  | 'awaiting_second'
  | 'contaminated'
  | 'triggered';

interface UseDoubleTapTriggerOptions {
  modifier: ModifierKey;
  intervalMs: number;
  enabled: boolean;
  onTrigger: (coords: { x: number; y: number }) => void;
}

function matchesModifier(key: string, modifier: ModifierKey): boolean {
  if (modifier === 'Command') return key === 'Meta';
  if (modifier === 'Control') return key === 'Control';
  if (modifier === 'Alt') return key === 'Alt';
  return false;
}

export function useDoubleTapTrigger({
  modifier,
  intervalMs,
  enabled,
  onTrigger,
}: UseDoubleTapTriggerOptions) {
  const [phase, setPhase] = useState<StateMachinePhase>('idle');
  const [lastEventNote, setLastEventNote] = useState<string>('');
  const [lastDeltaMs, setLastDeltaMs] = useState<number | null>(null);

  const mousePosRef = useRef<{ x: number; y: number }>({
    x: typeof window !== 'undefined' ? Math.round(window.innerWidth / 2) : 640,
    y: typeof window !== 'undefined' ? Math.round(window.innerHeight / 2) : 400,
  });

  const machineRef = useRef<{
    stage: 'idle' | 'holding_first' | 'awaiting_second' | 'holding_second';
    contaminated: boolean;
    firstReleaseAt: number;
    expireTimer: number | null;
    feedbackTimer: number | null;
  }>({
    stage: 'idle',
    contaminated: false,
    firstReleaseAt: 0,
    expireTimer: null,
    feedbackTimer: null,
  });

  const onTriggerRef = useRef(onTrigger);
  onTriggerRef.current = onTrigger;

  useEffect(() => {
    const clearExpire = () => {
      if (machineRef.current.expireTimer !== null) {
        window.clearTimeout(machineRef.current.expireTimer);
        machineRef.current.expireTimer = null;
      }
    };

    const flashPhase = (nextPhase: StateMachinePhase, note: string) => {
      if (machineRef.current.feedbackTimer !== null) {
        window.clearTimeout(machineRef.current.feedbackTimer);
      }
      setPhase(nextPhase);
      setLastEventNote(note);
      machineRef.current.feedbackTimer = window.setTimeout(() => {
        setPhase((cur) => (cur === nextPhase ? 'idle' : cur));
      }, 1200);
    };

    const resetMachine = (reason?: StateMachinePhase, note = '') => {
      clearExpire();
      machineRef.current.stage = 'idle';
      machineRef.current.contaminated = false;
      if (reason === 'contaminated') {
        flashPhase('contaminated', note);
      } else {
        setPhase('idle');
      }
    };

    if (!enabled) {
      resetMachine();
      return;
    }

    const handleMouseMove = (e: MouseEvent) => {
      mousePosRef.current = { x: e.clientX, y: e.clientY };
    };

    const handleMouseDown = () => {
      const m = machineRef.current;
      if (m.stage === 'holding_first' || m.stage === 'holding_second') {
        m.contaminated = true;
        resetMachine('contaminated', 'MouseClick + Modifier');
      } else if (m.stage === 'awaiting_second') {
        resetMachine('contaminated', 'MouseClick interrupted');
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.repeat) return;
      const m = machineRef.current;
      const isTarget = matchesModifier(e.key, modifier);

      if (isTarget) {
        if (m.stage === 'idle') {
          m.stage = 'holding_first';
          m.contaminated = false;
          setPhase('holding');
          setLastEventNote(`${modifier} Press (1/2)`);
        } else if (m.stage === 'awaiting_second') {
          clearExpire();
          m.stage = 'holding_second';
          m.contaminated = false;
          setPhase('holding');
          setLastEventNote(`${modifier} Press (2/2)`);
        }
      } else {
        // Any other key during modifier hold or awaiting second tap resets the machine
        if (m.stage === 'holding_first' || m.stage === 'holding_second') {
          m.contaminated = true;
          resetMachine('contaminated', `${modifier}+${e.key}`);
        } else if (m.stage === 'awaiting_second') {
          resetMachine('contaminated', `Interrupted by ${e.key}`);
        }
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      const m = machineRef.current;
      const isTarget = matchesModifier(e.key, modifier);
      if (!isTarget) return;

      const now = performance.now();

      if (m.stage === 'holding_first') {
        if (m.contaminated) {
          resetMachine('contaminated', 'Chord released');
          return;
        }
        m.firstReleaseAt = now;
        m.stage = 'awaiting_second';
        setPhase('awaiting_second');
        setLastEventNote(`${modifier} Release (1/2)`);
        clearExpire();
        m.expireTimer = window.setTimeout(() => {
          if (machineRef.current.stage === 'awaiting_second') {
            machineRef.current.stage = 'idle';
            setPhase('idle');
            setLastEventNote('Window expired');
          }
        }, intervalMs);
      } else if (m.stage === 'holding_second') {
        if (m.contaminated) {
          resetMachine('contaminated', 'Chord released');
          return;
        }
        const delta = Math.round(now - m.firstReleaseAt);
        setLastDeltaMs(delta);
        if (delta <= intervalMs) {
          clearExpire();
          m.stage = 'idle';
          m.contaminated = false;
          flashPhase('triggered', `Δ ${delta}ms ≤ ${intervalMs}ms`);
          onTriggerRef.current({ ...mousePosRef.current });
        } else {
          // Treat slow second release as the new first release
          m.firstReleaseAt = now;
          m.stage = 'awaiting_second';
          setPhase('awaiting_second');
          setLastEventNote(`Too slow (${delta}ms), restarted`);
          clearExpire();
          m.expireTimer = window.setTimeout(() => {
            if (machineRef.current.stage === 'awaiting_second') {
              machineRef.current.stage = 'idle';
              setPhase('idle');
            }
          }, intervalMs);
        }
      }
    };

    const handleBlur = () => {
      resetMachine();
    };

    window.addEventListener('mousemove', handleMouseMove, { passive: true });
    window.addEventListener('mousedown', handleMouseDown);
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    window.addEventListener('blur', handleBlur);

    return () => {
      clearExpire();
      if (machineRef.current.feedbackTimer !== null) {
        window.clearTimeout(machineRef.current.feedbackTimer);
      }
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mousedown', handleMouseDown);
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
      window.removeEventListener('blur', handleBlur);
    };
  }, [modifier, intervalMs, enabled]);

  return {
    phase,
    lastEventNote,
    lastDeltaMs,
    getPointerCoords: () => ({ ...mousePosRef.current }),
  };
}
