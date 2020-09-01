import Vue from 'vue';
import VueI18n from "vue-i18n";

const locales = {
  en: import('./langs/en')
}

Vue.use(VueI18n);

export async function loadLocale(i18n: VueI18n, lang) {
  const { default: messages } = await locales[lang];
  i18n.setLocaleMessage(lang, messages)
}

export const i18n = new VueI18n({
  locale: 'en',
  fallbackLocale: 'en',
})

// Load default locale
loadLocale(i18n, i18n.locale).catch(console.error);
