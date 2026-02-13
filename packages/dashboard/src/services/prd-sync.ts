// ABOUTME: Deterministic PRD progress synchronization from prd.json back to PRD markdown
// ABOUTME: Updates acceptance criteria checkboxes and adds implementation notes based on story completion

import type { PrdJson, UserStory } from './agent-runs';

// ── Types ──────────────────────────────────────────────────────────────────

export interface SyncResult {
  updatedMarkdown: string;
  storiesUpdated: number;
  storiesTotal: number;
}

// ── Sync Logic ─────────────────────────────────────────────────────────────

/**
 * Sync prd.json completion status back to PRD markdown.
 *
 * For each story with passes: true:
 * - Marks matching acceptance criteria checkboxes as [x]
 * - Appends implementation notes if present
 *
 * For stories with passes: false:
 * - Ensures checkboxes remain [ ] (unchecked)
 */
export function syncPrdProgress(
  prdJson: PrdJson,
  originalMarkdown: string,
): SyncResult {
  let markdown = originalMarkdown;
  let storiesUpdated = 0;

  for (const story of prdJson.userStories) {
    if (!story.passes) continue;

    // Try to find this story's section in the markdown by its ID or title
    const storyPattern = new RegExp(
      `(#{1,4}\\s*(?:${escapeRegex(story.id)}[:\\s-]*)?${escapeRegex(story.title)})`,
      'i',
    );

    const match = markdown.match(storyPattern);
    if (!match) {
      // Fall back: look for the story ID alone in a heading
      const idPattern = new RegExp(`(#{1,4}\\s*${escapeRegex(story.id)}\\b[^\\n]*)`, 'i');
      const idMatch = markdown.match(idPattern);
      if (!idMatch) continue;
    }

    // Check off acceptance criteria that match
    for (const criterion of story.acceptanceCriteria) {
      const criterionText = escapeRegex(criterion.replace(/^-\s*/, '').trim());
      // Match unchecked checkbox followed by the criterion text
      const checkboxPattern = new RegExp(
        `(- \\[)( )(\\]\\s*${criterionText})`,
        'i',
      );
      markdown = markdown.replace(checkboxPattern, '$1x$3');
    }

    // Append notes if present and not already in the markdown
    if (story.notes && story.notes.trim()) {
      const notesMarker = `<!-- ralph:notes:${story.id} -->`;
      if (!markdown.includes(notesMarker)) {
        // Find the end of this story's section (next heading of same or higher level, or EOF)
        const sectionEnd = findSectionEnd(markdown, story);
        if (sectionEnd !== -1) {
          const noteBlock = `\n\n${notesMarker}\n> **Implementation Notes (${story.id}):** ${story.notes.trim()}\n`;
          markdown = markdown.slice(0, sectionEnd) + noteBlock + markdown.slice(sectionEnd);
        }
      }
    }

    storiesUpdated++;
  }

  return {
    updatedMarkdown: markdown,
    storiesUpdated,
    storiesTotal: prdJson.userStories.length,
  };
}

// ── Helpers ────────────────────────────────────────────────────────────────

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function findSectionEnd(markdown: string, story: UserStory): number {
  // Find the story heading
  const headingPattern = new RegExp(
    `(#{1,4})\\s*(?:${escapeRegex(story.id)}[:\\s-]*)?${escapeRegex(story.title)}`,
    'i',
  );
  const headingMatch = markdown.match(headingPattern);
  if (!headingMatch || headingMatch.index === undefined) return -1;

  const headingLevel = headingMatch[1].length;
  const afterHeading = headingMatch.index + headingMatch[0].length;

  // Find the next heading of same or higher level
  const nextHeadingPattern = new RegExp(`\\n(#{1,${headingLevel}})\\s+`, 'g');
  nextHeadingPattern.lastIndex = afterHeading;
  const nextMatch = nextHeadingPattern.exec(markdown);

  return nextMatch ? nextMatch.index : markdown.length;
}
