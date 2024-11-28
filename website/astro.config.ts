import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLinksValidator from 'starlight-links-validator';

// https://astro.build/config
export default defineConfig({
  site: 'https://joinkitsune.org',
  integrations: [
    starlight({
      customCss: ['./src/styles/global.css'],
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
        mastodon: 'https://floss.social/@kitsune',
      },
      title: 'Kitsune',
    }),
  ],
});
