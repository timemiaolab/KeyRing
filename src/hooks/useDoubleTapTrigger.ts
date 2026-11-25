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

function matchesModifier(e: KeyboardEvent, modifier: ModifierKey): boolean {
  if (modifier === 'Command') {
    return (
      e.key === 'Meta' ||
      e.key === 'OS' ||
      e.code === 'MetaLeft' ||
      e.code === 'MetaRight'
    );
  }
  if (modifier === 'Control') {
    return (
      e.key === 'Control' ||
      e.code === 'ControlLeft' ||
      e.code === 'ControlRight'
    );
  }
  if (modifier === 'Alt') {
    return (
      e.key === 'Alt' ||
      e.key === 'AltGraph' ||
      e.code === 'AltLeft' ||
      e.code === 'AltRight'
    );
  }
  return false;
}

function isImeNoiseKey(e: KeyboardEvent): boolean {
  return (
    e.key === 'Process' ||
    e.key === 'Unidentified' ||
    e.key === 'Dead'
  );
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
    secondPressAt: number;
    expireTimer: number | null;
    feedbackTimer: number | null;
  }>({
    stage: 'idle',
    contaminated: false,
    firstReleaseAt: 0,
    secondPressAt: 0,
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
      }, 1400);
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

    const updateMousePos = (e: MouseEvent) => {
      if (e.clientX !== 0 || e.clientY !== 0) {
        mousePosRef.current = { x: e.clientX, y: e.clientY };
      }
    };

    const handleMouseDown = (e: MouseEvent) => {
      updateMousePos(e);
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
      const isTarget = matchesModifier(e, modifier);

      if (isTarget) {
        if (m.stage === 'idle' || m.stage === 'holding_first' || m.stage === 'holding_second') {
          // Self-heal if previous keyup was missed by OS/browser
          clearExpire();
          m.stage = 'holding_first';
          m.contaminated = false;
          setPhase('holding');
          setLastEventNote(`${modifier} Press (1/2)`);
        } else if (m.stage === 'awaiting_second') {
          clearExpire();
          m.stage = 'holding_second';
          m.secondPressAt = performance.now();
          m.contaminated = false;
          setPhase('holding');
          setLastEventNote(`${modifier} Press (2/2)`);
        }
      } else {
        // Do not let IME composition pseudo-keys falsely contaminate a clean modifier tap
        if (isImeNoiseKey(e)) return;

        // Any other real key during modifier hold or awaiting second tap resets the machine
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
      const isTarget = matchesModifier(e, modifier);
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
        const releaseDelta = Math.round(now - m.firstReleaseAt);
        const interTapGap = Math.round(m.secondPressAt - m.firstReleaseAt);
        const secondHoldMs = Math.round(now - m.secondPressAt);
        setLastDeltaMs(interTapGap);

        const withinWindow =
          releaseDelta <= intervalMs ||
          (interTapGap <= intervalMs && secondHoldMs <= 280);

        if (withinWindow) {
          clearExpire();
          m.stage = 'idle';
          m.contaminated = false;
          flashPhase('triggered', `Δ ${interTapGap}ms ≤ ${intervalMs}ms`);
          onTriggerRef.current({ ...mousePosRef.current });
        } else {
          // Treat slow second release as the new first release
          m.firstReleaseAt = now;
          m.stage = 'awaiting_second';
          setPhase('awaiting_second');
          setLastEventNote(`Too slow (${interTapGap}ms), restarted`);
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

    window.addEventListener('mousemove', updateMousePos, { capture: true, passive: true });
    window.addEventListener('mousedown', handleMouseDown, { capture: true });
    window.addEventListener('keydown', handleKeyDown, { capture: true });
    window.addEventListener('keyup', handleKeyUp, { capture: true });
    window.addEventListener('blur', handleBlur);

    return () => {
      clearExpire();
      if (machineRef.current.feedbackTimer !== null) {
        window.clearTimeout(machineRef.current.feedbackTimer);
      }
      window.removeEventListener('mousemove', updateMousePos, { capture: true });
      window.removeEventListener('mousedown', handleMouseDown, { capture: true });
      window.removeEventListener('keydown', handleKeyDown, { capture: true });
      window.removeEventListener('keyup', handleKeyUp, { capture: true });
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
