import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLinksValidator from 'starlight-links-validator';

// https://astro.build/config
export default defineConfig({
  site: 'https://joinkitsune.org',
  integrations: [
    starlight({
      customCss: ['./src/styles/global.scss'],
      plugins: [starlightLinksValidator()],
      sidebar: [
        {
          label: 'Run your own',
          autogenerate: { directory: 'running' },
        },
        {
          label: 'Configuration',
          autogenerate: { directory: 'configuration' },
        },
        {
          label: 'Specification',
          autogenerate: { directory: 'spec' },
        },
      ],
      social: {
        github: 'https://github.com/kitsune-soc/kitsune',
      },
      title: 'Kitsune',
    }),
  ],
});
