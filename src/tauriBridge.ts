export type WindowMode = 'pie' | 'config' | 'studio';

export interface RuntimeStatus {
  platform: string;
  accessibilityTrusted: boolean;
  listenerRunning: boolean;
  listenerError: string | null;
  paused: boolean;
  lastPlacementKind?: string | null;
  lastPlacementMessage?: string | null;
  lastPlacementClamp?: string | null;
}

export function getWindowMode(): WindowMode {
  if (typeof window === 'undefined') {
    return 'studio';
  }
  const value = new URLSearchParams(window.location.search).get('window');
  if (value === 'pie' || value === 'config') {
    return value;
  }
  return 'studio';
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(command, args);
  } catch {
    return null;
  }
}

export async function persistNativeConfig(config: unknown): Promise<void> {
  await invokeCommand('save_config', { nextConfig: config });
}

export async function fetchNativeConfig<T>(): Promise<T | null> {
  return invokeCommand<T>('get_config');
}

export async function hidePieWindow(): Promise<void> {
  await invokeCommand('hide_pie_window');
}

export async function executeNativeSlot(slotId: string): Promise<boolean> {
  const result = await invokeCommand('execute_slot', { slotId });
  return result !== null;
}

export async function fetchRuntimeStatus(): Promise<RuntimeStatus | null> {
  return invokeCommand<RuntimeStatus>('get_runtime_status');
}

export async function requestAccessibilityPermission(): Promise<RuntimeStatus | null> {
  return invokeCommand<RuntimeStatus>('request_accessibility_permission');
}

export async function openAccessibilitySettings(): Promise<void> {
  await invokeCommand('open_accessibility_settings_cmd');
}

export async function minimizeCurrentWindow(): Promise<void> {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().minimize();
  } catch {
    // not running inside Tauri
  }
}

export async function closeCurrentWindow(): Promise<void> {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().close();
  } catch {
    window.close();
  }
}

export async function listenNativeEvent(eventName: string, handler: () => void): Promise<() => void> {
  try {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen(eventName, () => handler());
    return unlisten;
  } catch {
    return () => undefined;
  }
}
