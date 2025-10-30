// ABOUTME: Reusable markdown renderer with proper sanitization and styling
// ABOUTME: Ensures consistent markdown rendering across all components with XSS protection

import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize, { defaultSchema } from 'rehype-sanitize';
import 'highlight.js/styles/github-dark.css';

interface MarkdownRendererProps {
  content: string;
  className?: string;
  includeHighlight?: boolean;
}

/**
 * Sanitization schema that allows markdown-generated HTML while protecting against XSS
 * Extends the default schema to allow className attributes for syntax highlighting
 */
const sanitizationSchema = {
  ...defaultSchema,
  attributes: {
    ...defaultSchema.attributes,
    code: [['className']],
    span: [['className']],
  },
};

/**
 * Reusable markdown renderer component with proper sanitization
 * Ensures consistent rendering across all components
 */
export function MarkdownRenderer({
  content,
  className = 'prose prose-sm dark:prose-invert max-w-none',
  includeHighlight = true,
}: MarkdownRendererProps) {
  const rehypePlugins = includeHighlight
    ? [rehypeHighlight, [rehypeSanitize, sanitizationSchema]]
    : [[rehypeSanitize, sanitizationSchema]];

  return (
    <div className={className}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={rehypePlugins}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}

export { sanitizationSchema };
