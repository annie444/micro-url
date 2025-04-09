import jsxa11y from "eslint-plugin-jsx-a11y";
import eslintPluginAstro from "eslint-plugin-astro";
import js from "@eslint/js";
import ts from "typescript-eslint";
import react from "eslint-plugin-react";
import globals from "globals";
import { globalIgnores } from "eslint/config";

export default ts.config([
  globalIgnores([
    "node_modules/",
    "**/node_modules",
    ".cargo/",
    "**/.cargo",
    ".nx/",
    "**/.nx",
    "apps/",
    "libs/",
    "kanidm/",
    "target/",
    "**/target",
    "dist/",
    "**/dist",
    ".astro/",
    "**/.astro",
  ]),
  // add more generic rule sets here, such as:
  js.configs.recommended,
  ts.configs.strict,
  ts.configs.stylistic,
  ...eslintPluginAstro.configs.recommended,
  {
    languageOptions: {
      globals: globals.browser,
    },
  },
  {
    rules: {
      // override/add rules settings here, such as:
      "astro/no-set-html-directive": "error",
    },
  },
  {
    files: ["**/*.{jsx,tsx}"],
    plugins: {
      react,
      jsxa11y,
    },
  },
]);
