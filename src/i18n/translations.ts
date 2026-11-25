import { SupportedLocale } from '../types/keyring';

export interface LocaleOption {
  code: SupportedLocale;
  nativeName: string;
  regionHint: string;
}

export const LOCALE_OPTIONS: LocaleOption[] = [
  { code: 'zh-CN', nativeName: '简体中文', regionHint: 'Default' },
  { code: 'zh-TW', nativeName: '繁體中文', regionHint: 'Taiwan / HK' },
  { code: 'en-US', nativeName: 'English', regionHint: 'United States' },
  { code: 'ja-JP', nativeName: '日本語', regionHint: 'Japan' },
  { code: 'ko-KR', nativeName: '한국어', regionHint: 'Korea' },
  { code: 'fr-FR', nativeName: 'Français', regionHint: 'France' },
  { code: 'de-DE', nativeName: 'Deutsch', regionHint: 'Germany' },
  { code: 'es-ES', nativeName: 'Español', regionHint: 'Spain' },
  { code: 'ru-RU', nativeName: 'Русский', regionHint: 'Russia' },
  { code: 'pt-BR', nativeName: 'Português', regionHint: 'Brazil' },
];

export interface TranslationDict {
  appName: string;
  workspaceTagline: string;
  nav: {
    composer: string;
    library: string;
    preferences: string;
    sandbox: string;
    source: string;
  };
  sidebar: {
    pagesGroup: string;
    runtimeGroup: string;
    testTriggerBtn: string;
    trayPreviewBtn: string;
    trayTitle: string;
    trayPause: string;
    trayResume: string;
    trayManage: string;
    trayAutostart: string;
    trayQuit: string;
    slotsCount: string;
    themeToggle: string;
    lightMode: string;
    darkMode: string;
  };
  builtins: {
    copy_path: string;
    copy_path_desc: string;
    open_in_terminal: string;
    open_in_terminal_desc: string;
  };
  directions: string[];
  composer: {
    title: string;
    subtitle: string;
    previewTitle: string;
    previewHint: string;
    popoutTest: string;
    activeListTitle: string;
    activeListDesc: string;
    emptyTitle: string;
    emptyDesc: string;
    goToLibrary: string;
    removeFromRing: string;
    moveUp: string;
    moveDown: string;
    dragHint: string;
    ringFullHint: string;
    slotDirectionLabel: string;
    centerIdleHint: string;
    copiedFeedback: string;
  };
  library: {
    title: string;
    subtitle: string;
    builtinSectionTitle: string;
    builtinSectionDesc: string;
    addedBadge: string;
    addToRing: string;
    removeFromRing: string;
    ringFullError: string;
    terminalPreferenceLabel: string;
    externalSectionTitle: string;
    externalSectionDesc: string;
    addCustomApp: string;
    scanCommonIdes: string;
    customSectionTitle: string;
    customSectionDesc: string;
    noCustomApps: string;
    deleteApp: string;
    modalTitle: string;
    modalNameLabel: string;
    modalPathLabel: string;
    modalPathPlaceholder: string;
    modalIconLabel: string;
    modalPickFile: string;
    modalCancel: string;
    modalConfirm: string;
    alreadyInLibrary: string;
    importAllMissing: string;
  };
  preferences: {
    title: string;
    subtitle: string;
    languageSection: string;
    languageLabel: string;
    languageDesc: string;
    triggerSection: string;
    modifierLabel: string;
    modifierDesc: string;
    intervalLabel: string;
    intervalDesc: string;
    antiMistouchTitle: string;
    antiMistouchDesc: string;
    stateIdle: string;
    stateHolding: string;
    stateAwaiting: string;
    stateContaminated: string;
    stateTriggered: string;
    appearanceSection: string;
    ringDiameterLabel: string;
    innerHoleLabel: string;
    iconSizeLabel: string;
    autostartLabel: string;
    autostartDesc: string;
    configSection: string;
    configDesc: string;
    copyConfig: string;
    downloadConfig: string;
    importConfig: string;
    resetConfig: string;
    configCopied: string;
  };
  sandbox: {
    title: string;
    subtitle: string;
    explorerHeader: string;
    currentDirLabel: string;
    selectedCountLabel: string;
    clearSelection: string;
    triggerHint: string;
    triggerNow: string;
    contextPayloadTitle: string;
    executionLogTitle: string;
    noExecutionsYet: string;
    terminalWindowTitle: string;
    closeTerminal: string;
  };
  source: {
    title: string;
    subtitle: string;
    copyFile: string;
    copied: string;
  };
  pie: {
    escHint: string;
    copiedToast: string;
    terminalLaunchedToast: string;
    appLaunchedToast: string;
    noActiveSlots: string;
  };
}

const zhCN: TranslationDict = {
  appName: 'KeyRing',
  workspaceTagline: '上下文感知圆环启动器',
  nav: {
    composer: '圆盘编排',
    library: '应用与动作库',
    preferences: '偏好设置',
    sandbox: '实时唤醒沙盒',
    source: 'Tauri v2 源码',
  },
  sidebar: {
    pagesGroup: '工作台',
    runtimeGroup: '系统托盘与测试',
    testTriggerBtn: '测试圆环弹出',
    trayPreviewBtn: '系统托盘菜单',
    trayTitle: 'KeyRing 托盘状态',
    trayPause: '暂停全局监听',
    trayResume: '恢复全局监听',
    trayManage: '管理 KeyRing',
    trayAutostart: '开机自启动',
    trayQuit: '退出 KeyRing',
    slotsCount: '已启用槽位',
    themeToggle: '外观主题',
    lightMode: '浅色',
    darkMode: '深色',
  },
  builtins: {
    copy_path: '复制文件路径',
    copy_path_desc: '获取选中文件或当前目录的绝对路径写入系统剪贴板，多选时以换行符分隔。',
    open_in_terminal: '用命令行打开',
    open_in_terminal_desc: '若选中文件夹则以该目录启动终端；若选中文件则在其父级目录打开终端。',
  },
  directions: ['正上', '右上', '正右', '右下', '正下', '左下', '正左', '左上'],
  composer: {
    title: '圆盘可视化编排',
    subtitle: '管理最多 8 个快捷槽位。圆环根据启用数量自动进行 360° / N 动态均分，悬停扇区可在中心镂空处预览名称。',
    previewTitle: '实时圆盘预览',
    previewHint: '悬停扇区查看中心提示，点击扇区可直接测试执行。',
    popoutTest: '以悬浮窗弹出测试',
    activeListTitle: '已启用槽位列表',
    activeListDesc: '拖拽条目调整方位顺序，或将其移回应用库。',
    emptyTitle: '圆盘暂无启用项目',
    emptyDesc: '请从“应用与动作库”中添加内置动作或常用 IDE。',
    goToLibrary: '前往应用与动作库',
    removeFromRing: '从圆盘移除',
    moveUp: '上移',
    moveDown: '下移',
    dragHint: '拖拽排序',
    ringFullHint: '已达 8 槽位上限',
    slotDirectionLabel: '方位',
    centerIdleHint: '悬停扇区',
    copiedFeedback: '已复制',
  },
  library: {
    title: '应用与动作库',
    subtitle: '内置系统动作与外部应用程序完全平权，可随时添加至圆盘、调整方位或移回此处。',
    builtinSectionTitle: '内置默认动作',
    builtinSectionDesc: '零剪贴板污染的上下文感知动作，出厂默认预置。',
    addedBadge: '已添加到圆盘',
    addToRing: '添加到圆盘',
    removeFromRing: '从圆盘移除',
    ringFullError: '圆盘已满（最多 8 个槽位），请先移除一个项目。',
    terminalPreferenceLabel: '目标终端适配器',
    externalSectionTitle: '常用 IDE 与工具一键导入',
    externalSectionDesc: '模拟后端扫描本机已安装的开发工具并自动提取高清图标与路径。',
    addCustomApp: '选取本地应用 / 自定义',
    scanCommonIdes: '重新扫描常用应用',
    customSectionTitle: '我的外部应用库',
    customSectionDesc: '已导入但当前未放在圆盘上的应用会保留在这里。',
    noCustomApps: '所有已导入的应用当前均已放置在圆盘上，或尚未导入自定义应用。',
    deleteApp: '彻底删除',
    modalTitle: '添加外部应用',
    modalNameLabel: '显示名称',
    modalPathLabel: '可执行文件路径 (.app / .exe / .desktop)',
    modalPathPlaceholder: '/Applications/Obsidian.app',
    modalIconLabel: '图标文件（可选，留空使用默认提取图标）',
    modalPickFile: '选择文件',
    modalCancel: '取消',
    modalConfirm: '添加到库',
    alreadyInLibrary: '已在库中',
    importAllMissing: '全部导入未添加项',
  },
  preferences: {
    title: '偏好设置',
    subtitle: '配置全局热键防误触状态机、圆环几何尺寸、界面语言及本地持久化 JSON。',
    languageSection: '界面与托盘语言',
    languageLabel: '显示语言（10 种语言实时生效）',
    languageDesc: '切换后配置界面、圆环中心悬停提示及系统托盘菜单将同步更新。',
    triggerSection: '全局触发与防误触机制',
    modifierLabel: '双击触发修饰键',
    modifierDesc: '记录完整的 Press → Release 周期；按下期间若有其他按键或鼠标点击将立即重置状态机。',
    intervalLabel: '双击判定间隔上限',
    intervalDesc: '两次独立 Release 之间的最大允许毫秒数（150ms ~ 500ms）。',
    antiMistouchTitle: '实时状态机监测器',
    antiMistouchDesc: '现在可以直接连续双击所选修饰键测试唤醒，或尝试按下修饰键+字母键（如 Cmd+C / Ctrl+C）验证防误触拦截。',
    stateIdle: '空闲等待 (Idle)',
    stateHolding: '修饰键按下中 (Holding)',
    stateAwaiting: '等待第二次点击 (Awaiting 2nd Tap)',
    stateContaminated: '检测到组合键/鼠标点击，已防误触重置',
    stateTriggered: '判定成功！已触发圆环唤起',
    appearanceSection: '圆环外观与系统行为',
    ringDiameterLabel: '圆环外径 (ringDiameter)',
    innerHoleLabel: '中心镂空直径 (innerHoleDiameter)',
    iconSizeLabel: '扇区图标尺寸 (iconSize)',
    autostartLabel: '开机自动启动',
    autostartDesc: '作为无 Dock / 任务栏图标的后台常驻应用随系统启动。',
    configSection: '本地配置文件 (config.json)',
    configDesc: '与 Tauri 后端持久化结构完全一致，支持实时复制或导出。',
    copyConfig: '复制 JSON',
    downloadConfig: '导出 config.json',
    importConfig: '导入 JSON',
    resetConfig: '恢复出厂默认',
    configCopied: '已复制 config.json 到剪贴板',
  },
  sandbox: {
    title: '实时唤醒沙盒',
    subtitle: '在模拟的文件管理器中单击或按住 Ctrl/Cmd 多选文件与文件夹，然后双击触发修饰键体验完整数据流。',
    explorerHeader: '模拟文件管理器（Finder / Explorer）',
    currentDirLabel: '当前激活目录',
    selectedCountLabel: '已选中项目',
    clearSelection: '清空选择（回退到当前目录）',
    triggerHint: '按下两次修饰键立即在鼠标位置唤起圆环',
    triggerNow: '在中心位置模拟唤起',
    contextPayloadTitle: '内存态 ContextState 快照',
    executionLogTitle: '动作执行日志',
    noExecutionsYet: '暂无执行记录。唤起圆环并点击任意扇区即可查看真实命令与剪贴板输出。',
    terminalWindowTitle: '模拟终端输出',
    closeTerminal: '关闭终端',
  },
  source: {
    title: 'Tauri v2 + Rust 跨平台源码',
    subtitle: '对应规格书第 3 章的完整工程模块（hotkey.rs、context.rs、icon.rs、executor.rs、tray.rs 等已生成至 src-tauri/）。',
    copyFile: '复制代码',
    copied: '已复制',
  },
  pie: {
    escHint: 'Esc 或点击空白处关闭',
    copiedToast: '路径已写入系统剪贴板',
    terminalLaunchedToast: '已在目标目录启动终端',
    appLaunchedToast: '已传入上下文路径拉起应用',
    noActiveSlots: '请先在编排器中添加至少 1 个项目',
  },
};

const zhTW: TranslationDict = {
  ...zhCN,
  workspaceTagline: '上下文感知圓環啟動器',
  nav: {
    composer: '圓盤編排',
    library: '應用與動作庫',
    preferences: '偏好設定',
    sandbox: '即時喚醒沙盒',
    source: 'Tauri v2 原始碼',
  },
  sidebar: {
    pagesGroup: '工作台',
    runtimeGroup: '系統匣與測試',
    testTriggerBtn: '測試圓環彈出',
    trayPreviewBtn: '系統匣選單',
    trayTitle: 'KeyRing 系統匣狀態',
    trayPause: '暫停全域監聽',
    trayResume: '恢復全域監聽',
    trayManage: '管理 KeyRing',
    trayAutostart: '開機自動啟動',
    trayQuit: '結束 KeyRing',
    slotsCount: '已啟用槽位',
    themeToggle: '外觀主題',
    lightMode: '淺色',
    darkMode: '深色',
  },
  builtins: {
    copy_path: '複製檔案路徑',
    copy_path_desc: '取得選取檔案或目前資料夾的絕對路徑寫入系統剪貼簿，多選時以換行符號分隔。',
    open_in_terminal: '以終端機開啟',
    open_in_terminal_desc: '若選取資料夾則以該目錄啟動終端機；若選取檔案則在其上層目錄開啟。',
  },
  composer: {
    ...zhCN.composer,
    title: '圓盤視覺化編排',
    subtitle: '管理最多 8 個捷徑槽位。圓環依據啟用數量自動進行 360° / N 等分，懸停扇區可在中心顯示名稱。',
    previewTitle: '即時圓盤預覽',
    popoutTest: '以懸浮窗彈出測試',
    activeListTitle: '已啟用槽位清單',
    removeFromRing: '從圓盤移除',
    goToLibrary: '前往應用與動作庫',
  },
  library: {
    ...zhCN.library,
    title: '應用與動作庫',
    builtinSectionTitle: '內建預設動作',
    addedBadge: '已新增至圓盤',
    addToRing: '新增至圓盤',
    removeFromRing: '從圓盤移除',
    externalSectionTitle: '常用 IDE 與工具一鍵匯入',
    addCustomApp: '選取本機應用 / 自訂',
  },
  preferences: {
    ...zhCN.preferences,
    title: '偏好設定',
    languageSection: '介面與系統匣語言',
    triggerSection: '全域觸發與防誤觸機制',
    appearanceSection: '圓環外觀與系統行為',
    autostartLabel: '開機自動啟動',
  },
};

const enUS: TranslationDict = {
  appName: 'KeyRing',
  workspaceTagline: 'Context-Aware Radial Launcher',
  nav: {
    composer: 'Ring Composer',
    library: 'App & Action Library',
    preferences: 'Preferences',
    sandbox: 'Live Trigger Sandbox',
    source: 'Tauri v2 Source',
  },
  sidebar: {
    pagesGroup: 'Workspace',
    runtimeGroup: 'Tray & Testing',
    testTriggerBtn: 'Test Pie Popup',
    trayPreviewBtn: 'System Tray Menu',
    trayTitle: 'KeyRing System Tray',
    trayPause: 'Pause Global Listener',
    trayResume: 'Resume Global Listener',
    trayManage: 'Manage KeyRing',
    trayAutostart: 'Launch at Login',
    trayQuit: 'Quit KeyRing',
    slotsCount: 'Active slots',
    themeToggle: 'Appearance',
    lightMode: 'Light',
    darkMode: 'Dark',
  },
  builtins: {
    copy_path: 'Copy File Path',
    copy_path_desc: 'Writes selected file/folder absolute paths to the system clipboard (newline-separated for multi-select), or falls back to the active folder.',
    open_in_terminal: 'Open in Terminal',
    open_in_terminal_desc: 'Opens your configured terminal in the selected folder, the parent directory of a selected file, or the active window folder.',
  },
  directions: ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'],
  composer: {
    title: 'Ring Composer',
    subtitle: 'Arrange up to 8 radial slots. Sectors automatically divide 360° / N based on active items, showing titles inside the center hole on hover.',
    previewTitle: 'Live Ring Preview',
    previewHint: 'Hover a sector to preview its center label, or click to execute.',
    popoutTest: 'Launch Floating Pie',
    activeListTitle: 'Active Ring Slots',
    activeListDesc: 'Drag items to reorder directions or remove them back to the library.',
    emptyTitle: 'No active slots on the ring',
    emptyDesc: 'Add built-in actions or applications from the App & Action Library.',
    goToLibrary: 'Open App & Action Library',
    removeFromRing: 'Remove from ring',
    moveUp: 'Move up',
    moveDown: 'Move down',
    dragHint: 'Drag to reorder',
    ringFullHint: '8-slot limit reached',
    slotDirectionLabel: 'Direction',
    centerIdleHint: 'Hover sector',
    copiedFeedback: 'Copied',
  },
  library: {
    title: 'App & Action Library',
    subtitle: 'Built-in actions and external applications share equal status. Add, remove, and reorder them freely.',
    builtinSectionTitle: 'Built-in Actions',
    builtinSectionDesc: 'Zero-clipboard-pollution context actions included out of the box.',
    addedBadge: 'Added to Ring',
    addToRing: 'Add to Ring',
    removeFromRing: 'Remove from Ring',
    ringFullError: 'The ring already has 8 slots. Remove one item first.',
    terminalPreferenceLabel: 'Target Terminal Adapter',
    externalSectionTitle: 'Quick-Import Common IDEs',
    externalSectionDesc: 'Detects installed editors and extracts high-resolution icons and bundle paths.',
    addCustomApp: 'Select Local App / Custom',
    scanCommonIdes: 'Rescan Common IDEs',
    customSectionTitle: 'Library Applications',
    customSectionDesc: 'Imported apps not currently placed on the ring remain stored here.',
    noCustomApps: 'All imported apps are currently active on the ring, or none added yet.',
    deleteApp: 'Delete',
    modalTitle: 'Add External Application',
    modalNameLabel: 'Display Name',
    modalPathLabel: 'Executable Path (.app / .exe / .desktop)',
    modalPathPlaceholder: '/Applications/Obsidian.app',
    modalIconLabel: 'Icon Image (optional)',
    modalPickFile: 'Choose File',
    modalCancel: 'Cancel',
    modalConfirm: 'Add to Library',
    alreadyInLibrary: 'In Library',
    importAllMissing: 'Import All Missing',
  },
  preferences: {
    title: 'Preferences',
    subtitle: 'Configure the double-tap anti-mistouch state machine, ring geometry, 10-language localization, and config.json.',
    languageSection: 'Language & Localization',
    languageLabel: 'Interface Language (10 Locales)',
    languageDesc: 'Updates configuration UI, radial center labels, and the Rust system tray menu immediately.',
    triggerSection: 'Global Hotkey & Anti-Mistouch',
    modifierLabel: 'Double-Tap Modifier Key',
    modifierDesc: 'Tracks complete Press → Release cycles. Any chorded key or mouse click during hold immediately resets the state machine.',
    intervalLabel: 'Double-Tap Interval Window',
    intervalDesc: 'Maximum allowed milliseconds between two clean Release events (150ms – 500ms).',
    antiMistouchTitle: 'Live State Machine Inspector',
    antiMistouchDesc: 'Double-tap your selected modifier key right now to test, or hold it with another key (e.g. Cmd+C / Ctrl+C) to verify chord rejection.',
    stateIdle: 'Idle',
    stateHolding: 'Holding Modifier',
    stateAwaiting: 'Awaiting 2nd Tap',
    stateContaminated: 'Chord / Mouse detected — Reset safely',
    stateTriggered: 'Double-tap confirmed — Pie triggered',
    appearanceSection: 'Ring Geometry & System',
    ringDiameterLabel: 'Ring Outer Diameter',
    innerHoleLabel: 'Center Hole Diameter',
    iconSizeLabel: 'Sector Icon Size',
    autostartLabel: 'Launch at System Startup',
    autostartDesc: 'Run silently as a background resident app without Dock or Taskbar icon.',
    configSection: 'Persistent Configuration (config.json)',
    configDesc: 'Matches the Tauri v2 backend schema byte-for-byte.',
    copyConfig: 'Copy JSON',
    downloadConfig: 'Export config.json',
    importConfig: 'Import JSON',
    resetConfig: 'Reset Defaults',
    configCopied: 'Copied config.json to clipboard',
  },
  sandbox: {
    title: 'Live Trigger Sandbox',
    subtitle: 'Select files or folders in the simulated file manager, then double-tap your modifier key anywhere to test the full context flow.',
    explorerHeader: 'Simulated File Manager (Finder / Explorer)',
    currentDirLabel: 'Active Folder',
    selectedCountLabel: 'Selected Items',
    clearSelection: 'Clear selection (use folder)',
    triggerHint: 'Double-tap your modifier key to summon the Pie Menu at cursor',
    triggerNow: 'Trigger at Center',
    contextPayloadTitle: 'In-Memory ContextState',
    executionLogTitle: 'Execution History',
    noExecutionsYet: 'No executions yet. Summon the Pie Menu and click any sector to see the real command and clipboard behavior.',
    terminalWindowTitle: 'Simulated Terminal Session',
    closeTerminal: 'Close Terminal',
  },
  source: {
    title: 'Tauri v2 + Rust Source Architecture',
    subtitle: 'Complete cross-platform backend modules (hotkey.rs, context.rs, icon.rs, executor.rs, tray.rs) generated in src-tauri/.',
    copyFile: 'Copy Code',
    copied: 'Copied',
  },
  pie: {
    escHint: 'Press Esc or click outside to dismiss',
    copiedToast: 'Path copied to system clipboard',
    terminalLaunchedToast: 'Terminal launched in target directory',
    appLaunchedToast: 'Launched application with context paths',
    noActiveSlots: 'Add at least 1 item in Ring Composer first',
  },
};

const jaJP: TranslationDict = {
  ...enUS,
  workspaceTagline: 'コンテキスト対応リングランチャー',
  nav: {
    composer: 'リング構成',
    library: 'アプリとアクション',
    preferences: '環境設定',
    sandbox: 'ライブテスト',
    source: 'Tauri v2 ソース',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: 'ワークスペース',
    runtimeGroup: 'トレイとテスト',
    testTriggerBtn: 'リングをテスト表示',
    trayPreviewBtn: 'トレイメニュー',
    trayTitle: 'KeyRing トレイ',
    trayPause: '監視を一時停止',
    trayResume: '監視を再開',
    trayManage: 'KeyRing を管理',
    trayAutostart: 'ログイン時に起動',
    trayQuit: 'KeyRing を終了',
    slotsCount: '有効スロット',
    themeToggle: '外観',
    lightMode: 'ライト',
    darkMode: 'ダーク',
  },
  builtins: {
    copy_path: 'ファイルパスをコピー',
    copy_path_desc: '選択したファイルまたはフォルダの絶対パスをクリップボードにコピーします。',
    open_in_terminal: 'ターミナルで開く',
    open_in_terminal_desc: '選択フォルダ（またはファイルの親ディレクトリ）でターミナルを起動します。',
  },
  directions: ['上', '右上', '右', '右下', '下', '左下', '左', '左上'],
  composer: {
    ...enUS.composer,
    title: 'リング構成エディタ',
    subtitle: '最大8つのスロットを配置します。有効数に応じて360°/Nで自動等分されます。',
    previewTitle: 'リアルタイムプレビュー',
    popoutTest: 'フローティング表示テスト',
    activeListTitle: '有効なスロット一覧',
    removeFromRing: 'リングから削除',
    goToLibrary: 'ライブラリを開く',
  },
  library: {
    ...enUS.library,
    title: 'アプリとアクションライブラリ',
    builtinSectionTitle: '組み込みアクション',
    addedBadge: '追加済み',
    addToRing: 'リングに追加',
    removeFromRing: 'リングから削除',
    externalSectionTitle: '主要 IDE クイックインポート',
    addCustomApp: 'ローカルアプリを追加',
  },
  preferences: {
    ...enUS.preferences,
    title: '環境設定',
    languageSection: '言語設定',
    triggerSection: 'ホットキーと誤操作防止',
    appearanceSection: 'リングサイズとシステム',
  },
};

const koKR: TranslationDict = {
  ...enUS,
  workspaceTagline: '컨텍스트 인식 원형 런처',
  nav: {
    composer: '링 구성',
    library: '앱 및 동작 라이브러리',
    preferences: '환경설정',
    sandbox: '실시간 샌드박스',
    source: 'Tauri v2 소스',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: '워크스페이스',
    runtimeGroup: '트레이 및 테스트',
    testTriggerBtn: '원형 메뉴 테스트',
    trayPreviewBtn: '시스템 트레이 메뉴',
    trayTitle: 'KeyRing 트레이',
    trayPause: '전역 감지 일시중지',
    trayResume: '전역 감지 재개',
    trayManage: 'KeyRing 관리',
    trayAutostart: '로그인 시 자동 실행',
    trayQuit: 'KeyRing 종료',
    slotsCount: '활성 슬롯',
    themeToggle: '테마',
    lightMode: '라이트',
    darkMode: '다크',
  },
  builtins: {
    copy_path: '파일 경로 복사',
    copy_path_desc: '선택한 파일 또는 현재 폴더의 절대 경로를 클립보드에 복사합니다.',
    open_in_terminal: '터미널에서 열기',
    open_in_terminal_desc: '선택한 폴더 또는 파일의 상위 디렉터리에서 터미널을 실행합니다.',
  },
  directions: ['상', '우상', '우', '우하', '하', '좌하', '좌', '좌상'],
  composer: {
    ...enUS.composer,
    title: '원형 링 구성',
    popoutTest: '플로팅 링 테스트',
    activeListTitle: '활성 슬롯 목록',
    removeFromRing: '링에서 제거',
  },
  library: {
    ...enUS.library,
    title: '앱 및 동작 라이브러리',
    builtinSectionTitle: '기본 내장 동작',
    addedBadge: '추가됨',
    addToRing: '링에 추가',
    removeFromRing: '링에서 제거',
  },
  preferences: {
    ...enUS.preferences,
    title: '환경설정',
    languageSection: '언어 설정',
    triggerSection: '전역 단축키 및 오작동 방지',
    appearanceSection: '외관 및 시스템',
  },
};

const frFR: TranslationDict = {
  ...enUS,
  workspaceTagline: 'Lanceur radial contextuel',
  nav: {
    composer: 'Éditeur d’anneau',
    library: 'Bibliothèque d’actions',
    preferences: 'Préférences',
    sandbox: 'Bac à sable',
    source: 'Code Tauri v2',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: 'Espace de travail',
    runtimeGroup: 'Barre d’état et test',
    testTriggerBtn: 'Tester le menu radial',
    trayPreviewBtn: 'Menu de la barre',
    trayTitle: 'KeyRing Barre système',
    trayPause: 'Suspendre l’écoute',
    trayResume: 'Reprendre l’écoute',
    trayManage: 'Gérer KeyRing',
    trayAutostart: 'Lancer au démarrage',
    trayQuit: 'Quitter KeyRing',
    slotsCount: 'Emplacements actifs',
    themeToggle: 'Apparence',
    lightMode: 'Clair',
    darkMode: 'Sombre',
  },
  builtins: {
    copy_path: 'Copier le chemin',
    copy_path_desc: 'Copie les chemins absolus sélectionnés dans le presse-papiers système.',
    open_in_terminal: 'Ouvrir dans le terminal',
    open_in_terminal_desc: 'Ouvre le terminal dans le dossier sélectionné ou le dossier parent.',
  },
  composer: {
    ...enUS.composer,
    title: 'Éditeur d’anneau',
    popoutTest: 'Tester l’anneau flottant',
    activeListTitle: 'Emplacements actifs',
    removeFromRing: 'Retirer de l’anneau',
  },
  library: {
    ...enUS.library,
    title: 'Bibliothèque d’applications et actions',
    builtinSectionTitle: 'Actions intégrées',
    addedBadge: 'Ajouté',
    addToRing: 'Ajouter à l’anneau',
    removeFromRing: 'Retirer',
  },
  preferences: {
    ...enUS.preferences,
    title: 'Préférences',
    languageSection: 'Langue de l’interface',
    triggerSection: 'Déclencheur global et anti-faux contact',
    appearanceSection: 'Géométrie et système',
  },
};

const deDE: TranslationDict = {
  ...enUS,
  workspaceTagline: 'Kontextsensitiver Ring-Launcher',
  nav: {
    composer: 'Ring-Konfigurator',
    library: 'App- & Aktionsbibliothek',
    preferences: 'Einstellungen',
    sandbox: 'Live-Sandbox',
    source: 'Tauri v2 Quellcode',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: 'Arbeitsbereich',
    runtimeGroup: 'Tray & Test',
    testTriggerBtn: 'Pie-Menü testen',
    trayPreviewBtn: 'System-Tray-Menü',
    trayTitle: 'KeyRing System-Tray',
    trayPause: 'Listener pausieren',
    trayResume: 'Listener fortsetzen',
    trayManage: 'KeyRing verwalten',
    trayAutostart: 'Beim Systemstart öffnen',
    trayQuit: 'KeyRing beenden',
    slotsCount: 'Aktive Slots',
    themeToggle: 'Erscheinungsbild',
    lightMode: 'Hell',
    darkMode: 'Dunkel',
  },
  builtins: {
    copy_path: 'Dateipfad kopieren',
    copy_path_desc: 'Kopiert ausgewählte absolute Pfade in die Systemzwischenablage.',
    open_in_terminal: 'Im Terminal öffnen',
    open_in_terminal_desc: 'Startet das Terminal im ausgewählten Ordner oder übergeordneten Verzeichnis.',
  },
  composer: {
    ...enUS.composer,
    title: 'Ring-Konfigurator',
    popoutTest: 'Schwebenden Ring testen',
    activeListTitle: 'Aktive Ring-Slots',
    removeFromRing: 'Aus Ring entfernen',
  },
  library: {
    ...enUS.library,
    title: 'App- & Aktionsbibliothek',
    builtinSectionTitle: 'Integrierte Aktionen',
    addedBadge: 'Im Ring aktiv',
    addToRing: 'Zum Ring hinzufügen',
    removeFromRing: 'Entfernen',
  },
  preferences: {
    ...enUS.preferences,
    title: 'Einstellungen',
    languageSection: 'Sprache & Lokalisierung',
    triggerSection: 'Globaler Trigger & Fehlauslösungsschutz',
    appearanceSection: 'Ring-Geometrie & System',
  },
};

const esES: TranslationDict = {
  ...enUS,
  workspaceTagline: 'Lanzador radial contextual',
  nav: {
    composer: 'Editor del anillo',
    library: 'Biblioteca de acciones',
    preferences: 'Preferencias',
    sandbox: 'Entorno de prueba',
    source: 'Código Tauri v2',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: 'Espacio de trabajo',
    runtimeGroup: 'Bandeja y pruebas',
    testTriggerBtn: 'Probar menú radial',
    trayPreviewBtn: 'Menú de bandeja',
    trayTitle: 'Bandeja de KeyRing',
    trayPause: 'Pausar escucha',
    trayResume: 'Reanudar escucha',
    trayManage: 'Administrar KeyRing',
    trayAutostart: 'Iniciar con el sistema',
    trayQuit: 'Salir de KeyRing',
    slotsCount: 'Ranuras activas',
    themeToggle: 'Apariencia',
    lightMode: 'Claro',
    darkMode: 'Oscuro',
  },
  builtins: {
    copy_path: 'Copiar ruta de archivo',
    copy_path_desc: 'Copia las rutas absolutas seleccionadas al portapapeles del sistema.',
    open_in_terminal: 'Abrir en terminal',
    open_in_terminal_desc: 'Abre la terminal en la carpeta seleccionada o en la carpeta principal.',
  },
  composer: {
    ...enUS.composer,
    title: 'Editor del anillo',
    popoutTest: 'Lanzar anillo flotante',
    activeListTitle: 'Ranuras activas',
    removeFromRing: 'Quitar del anillo',
  },
  library: {
    ...enUS.library,
    title: 'Biblioteca de apps y acciones',
    builtinSectionTitle: 'Acciones integradas',
    addedBadge: 'Añadido',
    addToRing: 'Añadir al anillo',
    removeFromRing: 'Quitar',
  },
  preferences: {
    ...enUS.preferences,
    title: 'Preferencias',
    languageSection: 'Idioma de la interfaz',
    triggerSection: 'Atajo global y antipulsación',
    appearanceSection: 'Geometría y sistema',
  },
};

const ruRU: TranslationDict = {
  ...enUS,
  workspaceTagline: 'Контекстный радиальный лаунчер',
  nav: {
    composer: 'Редактор кольца',
    library: 'Библиотека действий',
    preferences: 'Настройки',
    sandbox: 'Песочница',
    source: 'Код Tauri v2',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: 'Рабочая область',
    runtimeGroup: 'Трей и тест',
    testTriggerBtn: 'Тест меню-кольца',
    trayPreviewBtn: 'Меню в трее',
    trayTitle: 'Трей KeyRing',
    trayPause: 'Приостановить перехват',
    trayResume: 'Возобновить перехват',
    trayManage: 'Настроить KeyRing',
    trayAutostart: 'Автозапуск',
    trayQuit: 'Выйти из KeyRing',
    slotsCount: 'Активных слотов',
    themeToggle: 'Тема',
    lightMode: 'Светлая',
    darkMode: 'Тёмная',
  },
  builtins: {
    copy_path: 'Копировать путь',
    copy_path_desc: 'Записывает абсолютные пути выбранных файлов или текущей папки в буфер обмена.',
    open_in_terminal: 'Открыть в терминале',
    open_in_terminal_desc: 'Запускает терминал в выбранной папке или родительском каталоге файла.',
  },
  composer: {
    ...enUS.composer,
    title: 'Редактор кольца',
    popoutTest: 'Запустить кольцо',
    activeListTitle: 'Активные слоты',
    removeFromRing: 'Убрать с кольца',
  },
  library: {
    ...enUS.library,
    title: 'Библиотека приложений и действий',
    builtinSectionTitle: 'Встроенные действия',
    addedBadge: 'Добавлено',
    addToRing: 'Добавить на кольцо',
    removeFromRing: 'Убрать',
  },
  preferences: {
    ...enUS.preferences,
    title: 'Настройки',
    languageSection: 'Язык интерфейса',
    triggerSection: 'Глобальный триггер',
    appearanceSection: 'Геометрия и система',
  },
};

const ptBR: TranslationDict = {
  ...enUS,
  workspaceTagline: 'Lançador radial contextual',
  nav: {
    composer: 'Editor do Anel',
    library: 'Biblioteca de Ações',
    preferences: 'Preferências',
    sandbox: 'Sandbox ao Vivo',
    source: 'Código Tauri v2',
  },
  sidebar: {
    ...enUS.sidebar,
    pagesGroup: 'Área de Trabalho',
    runtimeGroup: 'Bandeja e Teste',
    testTriggerBtn: 'Testar Menu Radial',
    trayPreviewBtn: 'Menu da Bandeja',
    trayTitle: 'Bandeja KeyRing',
    trayPause: 'Pausar escuta global',
    trayResume: 'Retomar escuta',
    trayManage: 'Gerenciar KeyRing',
    trayAutostart: 'Iniciar com o sistema',
    trayQuit: 'Sair do KeyRing',
    slotsCount: 'Slots ativos',
    themeToggle: 'Aparência',
    lightMode: 'Claro',
    darkMode: 'Escuro',
  },
  builtins: {
    copy_path: 'Copiar Caminho',
    copy_path_desc: 'Copia os caminhos absolutos selecionados para a área de transferência.',
    open_in_terminal: 'Abrir no Terminal',
    open_in_terminal_desc: 'Abre o terminal na pasta selecionada ou no diretório pai do arquivo.',
  },
  composer: {
    ...enUS.composer,
    title: 'Editor do Anel',
    popoutTest: 'Testar Anel Flutuante',
    activeListTitle: 'Slots Ativos no Anel',
    removeFromRing: 'Remover do anel',
  },
  library: {
    ...enUS.library,
    title: 'Biblioteca de Apps e Ações',
    builtinSectionTitle: 'Ações Integradas',
    addedBadge: 'Adicionado',
    addToRing: 'Adicionar ao anel',
    removeFromRing: 'Remover',
  },
  preferences: {
    ...enUS.preferences,
    title: 'Preferências',
    languageSection: 'Idioma da Interface',
    triggerSection: 'Gatilho Global e Anti-Toque',
    appearanceSection: 'Geometria e Sistema',
  },
};

export const TRANSLATIONS: Record<SupportedLocale, TranslationDict> = {
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  'en-US': enUS,
  'ja-JP': jaJP,
  'ko-KR': koKR,
  'fr-FR': frFR,
  'de-DE': deDE,
  'es-ES': esES,
  'ru-RU': ruRU,
  'pt-BR': ptBR,
};
