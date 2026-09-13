import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  backendMessages,
  localeIds,
  messages,
  preference,
  resolveLocale,
  translate,
  translateBackend,
} from '../src/i18n.ts';

test('system language matching and explicit preferences', () => {
  const cases = [
    [['zh-Hans-CN'], 'zh-CN'],
    [['zh-Hant-TW'], 'zh-TW'],
    [['zh_TW'], 'zh-TW'],
    [['zh-HK'], 'zh-TW'],
    [['zh-Hans-HK'], 'zh-CN'],
    [['de-DE', 'zh-CN'], 'zh-CN'],
    [['en-US', 'zh-TW'], 'en'],
    [[], 'en'],
  ];
  for (const [languages, expected] of cases) {
    assert.equal(resolveLocale('auto', languages), expected);
  }
  for (const locale of localeIds) {
    assert.equal(resolveLocale(locale, []), locale, locale);
    assert.equal(preference(locale), locale, locale);
  }
  assert.equal(preference(null), 'auto');
  assert.equal(preference('auto'), 'auto');
  assert.equal(preference('zh-tw'), 'auto');
  assert.equal(preference('unexpected'), 'auto');
  assert.equal(preference('constructor'), 'auto');
});

test('every translation and HTML label has matching keys and placeholders', () => {
  assert.deepEqual(Object.keys(messages).sort(), [...localeIds].sort());
  const translated = localeIds.filter((locale) => locale !== 'en');
  const placeholders = (text) => [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
  for (const locale of translated) {
    assert.deepEqual(Object.keys(messages.en).sort(), Object.keys(messages[locale]).sort(), locale);
    for (const key of Object.keys(messages.en)) {
      assert.ok(messages[locale][key].trim(), `${locale}.${key}`);
      assert.deepEqual(
        placeholders(messages.en[key]),
        placeholders(messages[locale][key]),
        `${locale}.${key}`,
      );
    }
  }
  const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
  for (const [, key] of html.matchAll(/data-i18n(?:-aria-label)?="([^"]+)"/g)) {
    assert.ok(Object.hasOwn(messages.en, key), key);
  }
  const languageSelect = html.match(/<select id="language">([\s\S]*?)<\/select>/)?.[1] ?? '';
  const optionElements = [...languageSelect.matchAll(/<option\b([^>]*)>([\s\S]*?)<\/option>/g)].map(
    (match) => ({ value: match[1].match(/\bvalue="([^"]+)"/)?.[1], text: match[2].trim() }),
  );
  const languageOptions = optionElements.map((option) => option.value);
  assert.deepEqual([...languageOptions].sort(), ['auto', ...localeIds].sort());
  for (const option of optionElements) {
    assert.ok(option.text, `empty language option label: ${option.value}`);
    assert.ok(option.value === 'auto' || preference(option.value) === option.value, option.value);
  }
  assert.deepEqual(Object.keys(backendMessages).sort(), [...translated].sort());
  const [reference] = translated;
  for (const locale of translated) {
    assert.deepEqual(
      Object.keys(backendMessages[reference]).sort(),
      Object.keys(backendMessages[locale]).sort(),
      locale,
    );
    for (const [source, translation] of Object.entries(backendMessages[locale])) {
      assert.ok(translation.trim(), `${locale}: ${source}`);
      assert.deepEqual(placeholders(source), placeholders(translation), `${locale}: ${source}`);
    }
  }
});

test('dynamic values remain literal in every language', () => {
  assert.equal(translate('pair_remaining', 'en', { time: '4:03' }), 'Closes in 4:03');
  assert.equal(translate('pair_remaining', 'zh-CN', { time: '4:03' }), '4:03 后关闭');
  assert.equal(translate('pair_remaining', 'zh-TW', { time: '4:03' }), '4:03 後關閉');
  assert.equal(
    translate('last_connected', 'en', { date: '$& <script>' }),
    'Last connected: $& <script>',
  );
  assert.equal(translateBackend('Disabled in WebUI', 'zh-CN'), '已在 WebUI 中停用');
  assert.equal(translateBackend('Disabled in WebUI', 'zh-TW'), '已在 WebUI 中停用');
  assert.equal(translateBackend('Service started', 'en'), 'Service started');
  assert.equal(
    translateBackend('Unknown diagnostic $& <script>', 'zh-CN'),
    'Unknown diagnostic $& <script>',
  );
  assert.equal(
    translateBackend('Unknown diagnostic $& <script>', 'zh-TW'),
    'Unknown diagnostic $& <script>',
  );
  assert.equal(translateBackend('constructor', 'zh-CN'), 'constructor');
  assert.equal(
    translateBackend(
      'Cannot read system debugging state; service paused: Cannot determine native adbd state',
      'zh-CN',
    ),
    '无法读取系统调试状态，服务已暂停：无法确认原生 adbd 的运行状态',
  );
  assert.equal(
    translateBackend(
      'Cannot read system debugging state; service paused: Cannot determine native adbd state',
      'zh-TW',
    ),
    '無法讀取系統偵錯狀態，服務已暫停：無法確認系統 adbd 的執行狀態',
  );
  assert.equal(
    translateBackend('Cannot read system debugging state; service paused: $&', 'zh-CN'),
    '无法读取系统调试状态，服务已暂停：$&',
  );
  assert.equal(
    translateBackend('Cannot read system debugging state; service paused: $&', 'zh-TW'),
    '無法讀取系統偵錯狀態，服務已暫停：$&',
  );
});

test('CLI-owned status messages stay English', () => {
  for (const file of ['src/main.rs', 'src/unix/daemon.rs', 'src/unix/ksu.rs']) {
    const source = readFileSync(new URL(`../../${file}`, import.meta.url), 'utf8');
    assert.doesNotMatch(source, /\p{Script=Han}/u, file);
  }
});
