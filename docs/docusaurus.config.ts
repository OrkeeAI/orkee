import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Orkee',
  tagline: 'A CLI, TUI and dashboard for AI agent orchestration',
  favicon: 'img/favicon.ico',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://docs.orkee.ai',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'OrkeeAI', // Usually your GitHub org/user name.
  projectName: 'orkee', // Usually your repo name.

  onBrokenLinks: 'throw',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/OrkeeAI/orkee/tree/main/docs/',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: 'img/orkee-social-card.jpg',
    metadata: [
      {name: 'keywords', content: 'orkee, ai agent orchestration, cli, dashboard, tui, rust, typescript, project management'},
      {name: 'description', content: 'Comprehensive documentation for Orkee - A CLI, TUI and dashboard for AI agent orchestration'},
      {property: 'og:image', content: 'https://docs.orkee.ai/img/orkee-social-card.jpg'},
      {property: 'twitter:card', content: 'summary_large_image'},
    ],
    colorMode: {
      respectPrefersColorScheme: true,
      defaultMode: 'light',
      disableSwitch: false,
    },
    navbar: {
      title: 'Orkee',
      logo: {
        alt: 'Orkee Logo',
        src: 'img/logo.svg',
        srcDark: 'img/logo-dark.svg', // Optional dark mode logo
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://github.com/OrkeeAI/orkee',
          label: 'GitHub',
          position: 'right',
          className: 'header-github-link',
          'aria-label': 'GitHub repository',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Getting Started',
          items: [
            {
              label: 'Introduction',
              to: '/docs/intro',
            },
            {
              label: 'Installation',
              to: '/docs/getting-started/installation',
            },
            {
              label: 'Quick Start',
              to: '/docs/getting-started/quick-start',
            },
            {
              label: 'First Project',
              to: '/docs/getting-started/first-project',
            },
          ],
        },
        {
          title: 'Documentation',
          items: [
            {
              label: 'Configuration',
              to: '/docs/configuration/environment-variables',
            },
            {
              label: 'Security',
              to: '/docs/security/overview',
            },
            {
              label: 'API Reference',
              to: '/docs/api-reference/rest-api',
            },
            {
              label: 'Deployment',
              to: '/docs/deployment/docker',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub Issues',
              href: 'https://github.com/OrkeeAI/orkee/issues',
            },
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/OrkeeAI/orkee/discussions',
            },
            {
              label: 'Contributing',
              to: '/docs/development/contributing',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/OrkeeAI/orkee',
            },
            {
              label: 'Website',
              href: 'https://orkee.ai',
            },
            {
              label: 'Orkee Cloud',
              href: 'https://cloud.orkee.ai',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Orkee Team. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash', 'json', 'yaml', 'docker', 'nginx', 'sql'],
    },
    // algolia: {
    //   // Algolia search integration (to be configured later)
    //   appId: 'YOUR_APP_ID',
    //   apiKey: 'YOUR_SEARCH_API_KEY',
    //   indexName: 'orkee',
    // },
    tableOfContents: {
      minHeadingLevel: 2,
      maxHeadingLevel: 4,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
