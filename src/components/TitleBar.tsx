import React, { useEffect, useState } from 'react';
import { Copy, Minus, Square, X } from 'lucide-react';
import {
  closeCurrentWindow,
  detectDesktopPlatform,
  isCurrentWindowMaximized,
  minimizeCurrentWindow,
  toggleMaximizeCurrentWindow,
} from '../tauriBridge';

interface TitleBarProps {
  title?: string;
  platform?: string | null;
}

export const TitleBar: React.FC<TitleBarProps> = ({ title = 'KeyRing', platform }) => {
  const desktopPlatform = detectDesktopPlatform(platform);
  const isMac = desktopPlatform === 'macos';
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    let active = true;
    const syncMaximized = () => {
      void isCurrentWindowMaximized().then((val) => {
        if (active) setIsMaximized(val);
      });
    };
    syncMaximized();
    window.addEventListener('resize', syncMaximized);
    return () => {
      active = false;
      window.removeEventListener('resize', syncMaximized);
    };
  }, []);

  const handleToggleMaximize = () => {
    void toggleMaximizeCurrentWindow().then((val) => {
      setIsMaximized(val);
    });
  };

  if (isMac) {
    return (
      <div
        data-tauri-drag-region
        onDoubleClick={handleToggleMaximize}
        className="h-10 shrink-0 grid grid-cols-[1fr_auto_1fr] items-center px-3.5 border-b border-neutral-200/80 dark:border-neutral-800 bg-white dark:bg-[#1F1F1F] select-none"
      >
        {/* macOS traffic lights on the left: Close, Minimize, Zoom/Fullscreen */}
        <div
          className="flex items-center justify-start"
          onDoubleClick={(e) => e.stopPropagation()}
        >
          <div className="group/controls inline-flex items-center gap-2">
            <button
              type="button"
              aria-label="关闭"
              title="关闭"
              onMouseDown={(e) => e.stopPropagation()}
              onClick={() => void closeCurrentWindow()}
              className="w-3 h-3 rounded-full bg-[#FF5F57] border border-black/15 flex items-center justify-center text-[#4D0000] transition-transform active:scale-95"
            >
              <svg
                viewBox="0 0 8 8"
                className="w-2 h-2 opacity-0 group-hover/controls:opacity-100 transition-opacity"
                stroke="currentColor"
                strokeWidth="1.35"
                strokeLinecap="round"
              >
                <path d="M1.75 1.75L6.25 6.25M6.25 1.75L1.75 6.25" />
              </svg>
            </button>
            <button
              type="button"
              aria-label="最小化"
              title="最小化"
              onMouseDown={(e) => e.stopPropagation()}
              onClick={() => void minimizeCurrentWindow()}
              className="w-3 h-3 rounded-full bg-[#FEBC2E] border border-black/15 flex items-center justify-center text-[#5A3A00] transition-transform active:scale-95"
            >
              <svg
                viewBox="0 0 8 8"
                className="w-2 h-2 opacity-0 group-hover/controls:opacity-100 transition-opacity"
                stroke="currentColor"
                strokeWidth="1.35"
                strokeLinecap="round"
              >
                <path d="M1.5 4H6.5" />
              </svg>
            </button>
            <button
              type="button"
              aria-label={isMaximized ? '还原' : '全屏'}
              title={isMaximized ? '还原' : '全屏'}
              onMouseDown={(e) => e.stopPropagation()}
              onClick={handleToggleMaximize}
              className="w-3 h-3 rounded-full bg-[#28C840] border border-black/15 flex items-center justify-center text-[#004D00] transition-transform active:scale-95"
            >
              <svg
                viewBox="0 0 8 8"
                className="w-2 h-2 opacity-0 group-hover/controls:opacity-100 transition-opacity"
                fill="currentColor"
              >
                {isMaximized ? (
                  <path d="M1.5 4.8L3.2 4.8L3.2 6.5zM6.5 3.2L4.8 3.2L4.8 1.5z" />
                ) : (
                  <path d="M1.5 2L4.5 2L1.5 5zM6.5 6L3.5 6L6.5 3z" />
                )}
              </svg>
            </button>
          </div>
        </div>

        <span
          data-tauri-drag-region
          className="text-[12.5px] font-medium text-neutral-700 dark:text-neutral-200 truncate px-2"
        >
          {title}
        </span>

        <div data-tauri-drag-region className="h-full" />
      </div>
    );
  }

  return (
    <div
      data-tauri-drag-region
      onDoubleClick={handleToggleMaximize}
      className="h-10 shrink-0 flex items-center pl-3.5 pr-1 border-b border-neutral-200/80 dark:border-neutral-800 bg-white dark:bg-[#1F1F1F] select-none"
    >
      <div data-tauri-drag-region className="flex-1 flex items-center min-w-0">
        <span
          data-tauri-drag-region
          className="text-[12.5px] font-medium text-neutral-800 dark:text-neutral-100 truncate"
        >
          {title}
        </span>
      </div>

      {/* Windows / Linux window controls on the right: Minimize, Maximize/Restore, Close */}
      <div
        className="flex items-center h-full"
        onDoubleClick={(e) => e.stopPropagation()}
      >
        <button
          type="button"
          aria-label="最小化"
          title="最小化"
          onMouseDown={(e) => e.stopPropagation()}
          onClick={() => void minimizeCurrentWindow()}
          className="w-9 h-full flex items-center justify-center text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 hover:text-neutral-800 dark:hover:text-neutral-100 transition-colors"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          aria-label={isMaximized ? '还原' : '最大化'}
          title={isMaximized ? '还原' : '最大化'}
          onMouseDown={(e) => e.stopPropagation()}
          onClick={handleToggleMaximize}
          className="w-9 h-full flex items-center justify-center text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 hover:text-neutral-800 dark:hover:text-neutral-100 transition-colors"
        >
          {isMaximized ? (
            <Copy className="w-3 h-3 -scale-x-100" />
          ) : (
            <Square className="w-3 h-3" />
          )}
        </button>
        <button
          type="button"
          aria-label="关闭"
          title="关闭"
          onMouseDown={(e) => e.stopPropagation()}
          onClick={() => void closeCurrentWindow()}
          className="w-9 h-full flex items-center justify-center text-neutral-500 hover:bg-[#E81123] hover:text-white transition-colors"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};
