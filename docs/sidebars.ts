import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // Main documentation sidebar - only includes existing files
  tutorialSidebar: [
    // Introduction
    'intro',
    
    // Getting Started
    {
      type: 'category',
      label: 'Getting Started',
      link: {
        type: 'generated-index',
        title: 'Getting Started with Orkee',
        description: 'Learn how to install, configure, and create your first project with Orkee.',
      },
      collapsed: false,
      items: [
        {
          type: 'category',
          label: 'Installation',
          link: {
            type: 'doc',
            id: 'getting-started/installation',
          },
          items: [
            'getting-started/installation/npm-installation',
            'getting-started/installation/binary-installation',
            'getting-started/installation/source-installation',
            'getting-started/installation/troubleshooting',
          ],
        },
        'getting-started/quick-start',
        'getting-started/first-project',
      ],
    },

    // Configuration
    {
      type: 'category',
      label: 'Configuration',
      link: {
        type: 'generated-index',
        title: 'Configuration',
        description: 'Learn how to configure Orkee for your environment and requirements.',
      },
      items: [
        'configuration/environment-variables',
        'configuration/server-configuration',
        'configuration/security-settings',
        'configuration/cloud-sync',
      ],
    },

    // Deployment
    {
      type: 'category',
      label: 'Deployment',
      link: {
        type: 'generated-index',
        title: 'Deployment',
        description: 'Complete deployment guides for all platforms and environments.',
      },
      items: [
        'deployment/overview',
        'deployment/npm-installation',
        'deployment/docker',
        'deployment/binary-installation',
      ],
    },

    // API Reference
    {
      type: 'category',
      label: 'API Reference',
      link: {
        type: 'generated-index',
        title: 'API Reference',
        description: 'Complete REST API documentation for integrating with Orkee programmatically.',
      },
      items: [
        'api-reference/overview',
        'api-reference/health',
        'api-reference/projects',
        'api-reference/directories',
      ],
    },

    // Security
    {
      type: 'category',
      label: 'Security',
      link: {
        type: 'generated-index',
        title: 'Security',
        description: 'Comprehensive security documentation for Orkee deployment and operation.',
      },
      items: [
        'security/overview',
      ],
    },
  ],
};

export default sidebars;
