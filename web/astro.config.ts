import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: "Kitsune",
      social: {
        github: "https://github.com/kitsune-soc/kitsune",
      },
      sidebar: [
        {
          label: "Run your own",
          autogenerate: { directory: "running" },
        },
        {
          label: "Configuration",
          autogenerate: { directory: "configuration" },
        },
        {
          label: "Specification",
          autogenerate: { directory: "spec" },
        },
      ],
    }),
  ],
});
