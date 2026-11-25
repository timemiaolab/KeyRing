export type WindowMode = 'pie' | 'config' | 'studio';

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

async function invokeCommand(command: string, args?: Record<string, unknown>): Promise<boolean> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke(command, args);
    return true;
  } catch {
    return false;
  }
}

export async function persistNativeConfig(config: unknown): Promise<void> {
  await invokeCommand('save_config', { nextConfig: config });
}

export async function hidePieWindow(): Promise<void> {
  await invokeCommand('hide_pie_window');
}

export async function executeNativeSlot(slotId: string): Promise<boolean> {
  return invokeCommand('execute_slot', { slotId });
}
