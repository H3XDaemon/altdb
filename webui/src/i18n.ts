const english = {
  eyebrow: 'WIRELESS DEBUGGING',
  language: 'Language',
  language_auto: 'System',
  connecting: 'Connecting',
  connection_title: 'Device connection',
  reading_status: 'Reading service status…',
  connection_hint: 'Paired computers can discover this device or use the commands above.',
  connections: 'Active connections',
  host_count: 'Paired computers',
  pair_title: 'Pair a computer',
  pair_manual: 'Open manually',
  pair_instructions:
    'Run the pairing command on your computer, then enter the six-digit code below.',
  pair_code: 'Pairing code',
  pair_start: 'Start pairing',
  pair_stop: 'Stop pairing',
  pair_hint: 'Valid for five minutes. Closes after one successful pairing.',
  pair_remaining: 'Closes in {time}',
  settings: 'Service settings',
  enable: 'Enable altdb',
  enable_hint: 'Pauses while system debugging is enabled and resumes when it is off',
  port: 'Connection port',
  port_random: 'Random port',
  port_fixed: 'Fixed port',
  fixed_port: 'Port number',
  port_hint: 'Random ports change when the service restarts or recovers from a conflict.',
  adb_root: 'Allow adb root',
  adb_root_hint: 'Computers can switch all connections to root; restarting restores shell',
  shell_root: 'Allow shell to use su',
  shell_root_hint: 'KernelSU must still authorize root access for shell',
  save: 'Save settings',
  hosts: 'Paired computers',
  local_auth: 'Device authorizations',
  no_hosts: 'No paired computers',
  logs: 'Diagnostics',
  refresh: 'Refresh',
  logs_prompt: 'Refresh to view service events',
  no_logs: 'No service events',
  footer: 'TLS-encrypted connections',
  state_running: 'Running',
  state_paused_native: 'System debugging',
  state_disabled: 'Disabled',
  state_error: 'Needs attention',
  state_waiting_network: 'Waiting for network',
  state_starting: 'Starting',
  tls_only: 'Only TLS connections from paired computers are accepted.',
  unavailable: 'Service unavailable',
  encoding_failed: 'Could not encode the request.',
  no_service:
    'The service is not running. Check that the module is enabled, then run altdb doctor.',
  invalid_reply: 'The service returned an invalid response.',
  failed: 'Operation failed',
  error_detail: 'Operation failed: {detail}',
  updated: 'Updated',
  copied: 'Command copied',
  copy_failed: 'Please copy the command manually',
  copy: 'Copy',
  last_connected: 'Last connected: {date}',
  never_connected: 'Never connected',
  revoke: 'Revoke',
  invalid_port: 'Port must be an integer from 1024 to 65535.',
} as const;

export type MessageKey = keyof typeof english;
export const localeIds = ['en', 'zh-CN', 'zh-TW'] as const;
Object.freeze(localeIds);
export type Locale = (typeof localeIds)[number];
export type LanguagePreference = 'auto' | Locale;
export const languageStorageKey = 'altdb.language';
export const messages = {
  en: english,
  'zh-CN': {
    eyebrow: '无线调试',
    language: '语言',
    language_auto: '跟随系统',
    connecting: '连接中',
    connection_title: '设备连接',
    reading_status: '正在读取服务状态…',
    connection_hint: '已配对的电脑可自动发现，也可使用上方命令连接。',
    connections: '当前连接',
    host_count: '已配对电脑',
    pair_title: '配对新电脑',
    pair_manual: '仅手动开启',
    pair_instructions: '在电脑执行配对命令，再输入下方六位配对码。',
    pair_code: '配对码',
    pair_start: '开始配对',
    pair_stop: '结束配对',
    pair_hint: '五分钟内有效，成功配对一台电脑后自动关闭。',
    pair_remaining: '{time} 后关闭',
    settings: '服务设置',
    enable: '启用 altdb',
    enable_hint: '原生调试开启时暂停，关闭后自动恢复',
    port: '通信端口',
    port_random: '随机端口',
    port_fixed: '固定端口',
    fixed_port: '固定端口号',
    port_hint: '随机端口在服务重启或冲突恢复时更新。',
    adb_root: '允许 adb root',
    adb_root_hint: '电脑可将全部连接切换为 root；服务重启后恢复 shell',
    shell_root: '允许 shell 使用 su',
    shell_root_hint: '开启后，仍需要 KernelSU 授权 shell 提权',
    save: '保存设置',
    hosts: '已配对电脑',
    local_auth: '本机授权',
    no_hosts: '尚无配对记录',
    logs: '诊断日志',
    refresh: '刷新',
    logs_prompt: '点击刷新查看服务事件',
    no_logs: '暂无服务事件',
    footer: 'TLS 加密连接',
    state_running: '运行中',
    state_paused_native: '原生调试优先',
    state_disabled: '已停用',
    state_error: '需要处理',
    state_waiting_network: '等待网络',
    state_starting: '启动中',
    tls_only: '仅接受已配对电脑的 TLS 连接。',
    unavailable: '服务不可用',
    encoding_failed: '请求编码失败。',
    no_service: '服务未运行。请检查模块是否启用，并运行 altdb doctor。',
    invalid_reply: '服务返回了无效响应。',
    failed: '操作失败',
    error_detail: '操作失败：{detail}',
    updated: '已更新',
    copied: '命令已复制',
    copy_failed: '请手动复制命令',
    copy: '复制',
    last_connected: '最近连接：{date}',
    never_connected: '尚未连接',
    revoke: '撤销',
    invalid_port: '端口必须为 1024–65535 之间的整数。',
  } satisfies Record<MessageKey, string>,
  'zh-TW': {
    eyebrow: '無線偵錯',
    language: '語言',
    language_auto: '跟隨系統',
    connecting: '連線中',
    connection_title: '裝置連線',
    reading_status: '正在讀取服務狀態…',
    connection_hint: '已配對的電腦可自動探索此裝置，也可使用上方指令連線。',
    connections: '目前連線',
    host_count: '已配對電腦',
    pair_title: '配對新電腦',
    pair_manual: '手動開啟',
    pair_instructions: '請在電腦上執行配對指令，然後輸入下方的六位數配對碼。',
    pair_code: '配對碼',
    pair_start: '開始配對',
    pair_stop: '停止配對',
    pair_hint: '五分鐘內有效，成功配對一台電腦後自動關閉。',
    pair_remaining: '{time} 後關閉',
    settings: '服務設定',
    enable: '啟用 altdb',
    enable_hint: '系統偵錯開啟時暫停，關閉後自動恢復',
    port: '連接埠',
    port_random: '隨機連接埠',
    port_fixed: '固定連接埠',
    fixed_port: '連接埠號碼',
    port_hint: '服務重新啟動或從衝突中恢復時，隨機連接埠會變更。',
    adb_root: '允許 adb root',
    adb_root_hint: '電腦可將所有連線切換為 root；服務重新啟動後恢復 shell',
    shell_root: '允許 shell 使用 su',
    shell_root_hint: '開啟後，仍須由 KernelSU 授權 shell 取得 root 權限',
    save: '儲存設定',
    hosts: '已配對電腦',
    local_auth: '本機授權',
    no_hosts: '尚未配對任何電腦',
    logs: '診斷日誌',
    refresh: '重新整理',
    logs_prompt: '點選重新整理以查看服務事件',
    no_logs: '尚無服務事件',
    footer: 'TLS 加密連線',
    state_running: '執行中',
    state_paused_native: '系統偵錯優先',
    state_disabled: '已停用',
    state_error: '需要處理',
    state_waiting_network: '等待網路',
    state_starting: '啟動中',
    tls_only: '僅接受來自已配對電腦的 TLS 連線。',
    unavailable: '服務無法使用',
    encoding_failed: '無法編碼此要求。',
    no_service: '服務未執行。請確認模組已啟用，然後執行 altdb doctor。',
    invalid_reply: '服務傳回無效的回應。',
    failed: '操作失敗',
    error_detail: '操作失敗：{detail}',
    updated: '已更新',
    copied: '指令已複製',
    copy_failed: '請手動複製指令',
    copy: '複製',
    last_connected: '上次連線：{date}',
    never_connected: '從未連線',
    revoke: '撤銷',
    invalid_port: '連接埠必須為 1024–65535 之間的整數。',
  } satisfies Record<MessageKey, string>,
} satisfies Record<Locale, Record<MessageKey, string>>;

const isLocale = (value: string): value is Locale => localeIds.some((locale) => locale === value);

export function preference(value: string | null): LanguagePreference {
  return typeof value === 'string' && isLocale(value) ? value : 'auto';
}

export function resolveLocale(choice: LanguagePreference, languages: readonly string[]): Locale {
  if (choice !== 'auto') return choice;
  for (const language of languages) {
    const [tag, ...subtags] = language.toLowerCase().replaceAll('_', '-').split('-');
    if (tag === 'zh') {
      const traditional =
        subtags.includes('hant') ||
        (!subtags.includes('hans') &&
          (subtags.includes('tw') || subtags.includes('hk') || subtags.includes('mo')));
      return traditional ? 'zh-TW' : 'zh-CN';
    }
    if (tag === 'en') return 'en';
  }
  return 'en';
}

export function translate(
  key: MessageKey,
  locale: Locale,
  params: Record<string, string | number> = {},
): string {
  return messages[locale][key].replace(/\{(\w+)\}/g, (token, name: string) =>
    String(params[name] ?? token),
  );
}

// English service messages are the message IDs. Unknown diagnostics remain
// verbatim, and user-provided host names are never sent through this catalog.
export const backendMessages: Record<Exclude<Locale, 'en'>, Record<string, string>> = {
  'zh-CN': {
    'Debugging settings are temporarily unavailable; keeping connections':
      '调试设置暂时无法读取，保持现有连接',
    'Debugging settings are available again': '调试设置读取已恢复',
    'Checking network worker permissions': '正在检查网络进程权限',
    'Service is not running': '服务当前未运行',
    'Paired host limit reached; revoke a host first': '请先撤销一个主机授权，已达到配对数量上限',
    'Pairing window opened': '配对窗口已开启',
    'Pairing window closed': '配对窗口已关闭',
    'Configuration updated; checking startup conditions': '配置已更新，正在重新检查启动条件',
    'Configuration saved': '配置已保存',
    'Host authorization revoked': '主机授权已撤销',
    'Service stopped': '服务已停止',
    'Paired host connected': '已配对主机已连接',
    'Host paired successfully': '主机配对成功',
    'Pairing authentication failed': '配对认证失败',
    'Pairing window expired': '配对窗口已到期',
    'Service failed to start after permission change': '权限切换后服务启动失败',
    'Service started': '服务已启动',
    'Network worker communication failed': '网络进程通信失败',
    'Network worker exited; check SELinux and KernelSU':
      '网络进程已退出，请检查 SELinux 与 KernelSU',
    'Disabled in WebUI': '已在 WebUI 中停用',
    'Cannot read system debugging state; service paused: {error}':
      '无法读取系统调试状态，服务已暂停：{error}',
    'Waiting for Wi-Fi, hotspot or Ethernet': '等待 Wi-Fi、热点或有线局域网',
    'Network changed; starting service': '网络已变化，正在启动服务',
    'Cannot bind port or publish mDNS; check port availability and interface permissions':
      '无法绑定端口或发布 mDNS，请检查端口占用和接口权限',
    'Cannot read network interfaces': '无法读取网络接口',
    'Native wireless debugging is enabled': '原生无线调试已开启',
    'Native USB debugging is enabled': '原生 USB 调试已开启',
    'Native adbd is running or stopping': '原生 adbd 正在运行或停止中',
    'Cannot determine native adbd state': '无法确认原生 adbd 的运行状态',
    'Cannot read native debugging settings or processes; service paused: {error}':
      '无法读取原生调试设置或进程，服务已暂停：{error}',
    'Checking native debugging settings and processes': '正在检查原生调试设置和进程',
    'Native debugging check is stale; service paused': '原生调试检测结果已过期，服务已暂停',
    'A running native adbd process was detected': '检测到正在运行的原生 adbd 进程',
    'Timed out reading debugging settings': '读取调试设置超时',
    'Cannot read system debugging settings': '无法读取系统调试设置',
    'System debugging settings are unavailable': '系统调试设置不可用',
    'Native debugging is enabled in developer options': '系统开发者选项已开启原生调试',
    'Unrecognized native debugging setting': '原生调试开关值不可识别',
    'fixed port must be 1024–65535': '固定端口必须为 1024–65535',
    'pairing window closed': '配对窗口已关闭',
    'pairing window expired': '配对窗口已到期',
    'host is no longer authorized': '主机授权已被撤销',
    'root required': '需要 root 权限',
    'adb root is disabled in altdb': 'altdb 已禁用 adb root',
    'network worker unavailable': '网络进程不可用',
    'connection limit reached': '已达到连接数量上限',
  },
  'zh-TW': {
    'Debugging settings are temporarily unavailable; keeping connections':
      '偵錯設定暫時無法讀取，保持現有連線',
    'Debugging settings are available again': '偵錯設定讀取已恢復',
    'Checking network worker permissions': '正在檢查網路工作程序權限',
    'Service is not running': '服務目前未執行',
    'Paired host limit reached; revoke a host first': '已達到配對數量上限，請先撤銷一台電腦的授權',
    'Pairing window opened': '配對視窗已開啟',
    'Pairing window closed': '配對視窗已關閉',
    'Configuration updated; checking startup conditions': '設定已更新，正在重新檢查啟動條件',
    'Configuration saved': '設定已儲存',
    'Host authorization revoked': '電腦授權已撤銷',
    'Service stopped': '服務已停止',
    'Paired host connected': '已配對的電腦已連線',
    'Host paired successfully': '電腦配對成功',
    'Pairing authentication failed': '配對驗證失敗',
    'Pairing window expired': '配對視窗已過期',
    'Service failed to start after permission change': '權限切換後服務啟動失敗',
    'Service started': '服務已啟動',
    'Network worker communication failed': '網路工作程序通訊失敗',
    'Network worker exited; check SELinux and KernelSU':
      '網路工作程序已結束，請檢查 SELinux 與 KernelSU',
    'Disabled in WebUI': '已在 WebUI 中停用',
    'Cannot read system debugging state; service paused: {error}':
      '無法讀取系統偵錯狀態，服務已暫停：{error}',
    'Waiting for Wi-Fi, hotspot or Ethernet': '等待 Wi-Fi、熱點或乙太網路',
    'Network changed; starting service': '網路已變更，正在啟動服務',
    'Cannot bind port or publish mDNS; check port availability and interface permissions':
      '無法綁定連接埠或發布 mDNS，請檢查連接埠是否可用，並確認介面權限',
    'Cannot read network interfaces': '無法讀取網路介面',
    'Native wireless debugging is enabled': '系統無線偵錯已開啟',
    'Native USB debugging is enabled': '系統 USB 偵錯已開啟',
    'Native adbd is running or stopping': '系統 adbd 正在執行或停止中',
    'Cannot determine native adbd state': '無法確認系統 adbd 的執行狀態',
    'Cannot read native debugging settings or processes; service paused: {error}':
      '無法讀取系統偵錯設定或程序，服務已暫停：{error}',
    'Checking native debugging settings and processes': '正在檢查系統偵錯設定與程序',
    'Native debugging check is stale; service paused': '系統偵錯檢查結果已過期，服務已暫停',
    'A running native adbd process was detected': '偵測到正在執行的系統 adbd 程序',
    'Timed out reading debugging settings': '讀取偵錯設定逾時',
    'Cannot read system debugging settings': '無法讀取系統偵錯設定',
    'System debugging settings are unavailable': '系統偵錯設定無法使用',
    'Native debugging is enabled in developer options': '開發人員選項中已開啟系統偵錯',
    'Unrecognized native debugging setting': '無法辨識的系統偵錯設定值',
    'fixed port must be 1024–65535': '固定連接埠必須為 1024–65535',
    'pairing window closed': '配對視窗已關閉',
    'pairing window expired': '配對視窗已過期',
    'host is no longer authorized': '電腦授權已失效',
    'root required': '需要 root 權限',
    'adb root is disabled in altdb': 'altdb 已停用 adb root',
    'network worker unavailable': '網路工作程序無法使用',
    'connection limit reached': '已達到連線數量上限',
  },
};

export function translateBackend(message: string, locale: Locale, depth = 0): string {
  if (locale === 'en' || depth >= 4) return message;
  const table = backendMessages[locale];
  if (Object.hasOwn(table, message)) return table[message];
  for (const [source, translation] of Object.entries(table)) {
    if (!source.endsWith('{error}')) continue;
    const prefix = source.slice(0, -'{error}'.length);
    if (message.startsWith(prefix)) {
      return translation.replace('{error}', () =>
        translateBackend(message.slice(prefix.length), locale, depth + 1),
      );
    }
  }
  return message;
}
