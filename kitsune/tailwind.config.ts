import forms from "@tailwindcss/forms";
import typography from "@tailwindcss/typography";
import { extendTheme } from "./theme";

import type { Config } from "tailwindcss";

export default {
  content: ["./templates/**/*.{html,js,ts}"],

  theme: {
    extend: {
      ...extendTheme,
    },
  },

  plugins: [typography, forms],
} as Config;
