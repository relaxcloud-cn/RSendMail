import { createI18n } from 'vue-i18n'
import enUS from '../../rsendmail-i18n/locales/en-US.yml'
import jaJP from '../../rsendmail-i18n/locales/ja-JP.yml'
import koKR from '../../rsendmail-i18n/locales/ko-KR.yml'
import zhCN from '../../rsendmail-i18n/locales/zh-CN.yml'
import zhTW from '../../rsendmail-i18n/locales/zh-TW.yml'

const supportedLocales = ['en-US', 'ja-JP', 'ko-KR', 'zh-CN', 'zh-TW'] as const
type SupportedLocale = typeof supportedLocales[number]

function normalizeLocale(locale: string | null | undefined): SupportedLocale {
  switch (locale) {
    case 'en':
    case 'en-US':
      return 'en-US'
    case 'ja':
    case 'ja-JP':
      return 'ja-JP'
    case 'ko':
    case 'ko-KR':
      return 'ko-KR'
    case 'zh':
    case 'zh-CN':
    case 'zh-Hans':
      return 'zh-CN'
    case 'zh-TW':
    case 'zh-HK':
    case 'zh-MO':
    case 'zh-Hant':
      return 'zh-TW'
    default:
      return 'en-US'
  }
}

function resolveInitialLocale(): SupportedLocale {
  if (typeof window !== 'undefined') {
    const savedLocale = window.localStorage.getItem('rsendmail-locale')
    if (savedLocale && supportedLocales.includes(savedLocale as SupportedLocale)) {
      return savedLocale as SupportedLocale
    }

    return normalizeLocale(window.navigator.language)
  }

  return 'en-US'
}

const i18n = createI18n({
  legacy: false, // You must set `false`, to use Composition API
  locale: resolveInitialLocale(),
  fallbackLocale: 'en-US', // fallback locale
  messages: {
    'en-US': enUS,
    'ja-JP': jaJP,
    'ko-KR': koKR,
    'zh-CN': zhCN,
    'zh-TW': zhTW,
  }
})

export default i18n
