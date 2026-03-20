import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import common from "./locales/zh-CN/common.json";
import diagnostics from "./locales/zh-CN/diagnostics.json";
import overview from "./locales/zh-CN/overview.json";
import settings from "./locales/zh-CN/settings.json";
import setup from "./locales/zh-CN/setup.json";

void i18n.use(initReactI18next).init({
  lng: "zh-CN",
  fallbackLng: "zh-CN",
  interpolation: {
    escapeValue: false
  },
  resources: {
    "zh-CN": {
      common,
      overview,
      setup,
      settings,
      diagnostics
    }
  }
});

export default i18n;
