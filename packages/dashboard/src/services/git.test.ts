// ABOUTME: Tests for Git service API functions
// ABOUTME: Validates commit history, commit details, and file diff retrieval with proper response handling

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  getCommitHistory,
  getCommitDetails,
  getFileDiff,
  formatCommitMessage,
  formatAuthor,
  formatFileStatus,
  type CommitInfo,
  type CommitDetail,
  type FileDiff,
} from './git';
import { apiRequest } from './api';

// Mock the api module
vi.mock('./api', () => ({
  apiRequest: vi.fn(),
}));

describe('Git Service', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getCommitHistory', () => {
    const mockCommits: CommitInfo[] = [
      {
        id: 'abc123def456',
        short_id: 'abc123d',
        message: 'Initial commit',
        author: 'Test User',
        email: 'test@example.com',
        date: '2025-01-15 12:00:00 UTC',
        timestamp: 1736942400,
        files_changed: 3,
        insertions: 50,
        deletions: 10,
      },
      {
        id: 'def456ghi789',
        short_id: 'def456g',
        message: 'Add new feature',
        author: 'Another User',
        email: 'another@example.com',
        date: '2025-01-16 14:30:00 UTC',
        timestamp: 1737037800,
        files_changed: 2,
        insertions: 25,
        deletions: 5,
      },
    ];

    it('should return commit array directly from response.data', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockCommits,
        error: undefined,
      });

      const result = await getCommitHistory('proj-123');

      expect(result).toEqual(mockCommits);
      expect(result).toBeInstanceOf(Array);
      expect(result).toHaveLength(2);
      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/commits',
        { method: 'GET' }
      );
    });

    it('should handle pagination parameters correctly', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockCommits,
        error: undefined,
      });

      await getCommitHistory('proj-123', { page: 2, per_page: 25 });

      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/commits?page=2&per_page=25',
        { method: 'GET' }
      );
    });

    it('should handle branch parameter correctly', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockCommits,
        error: undefined,
      });

      await getCommitHistory('proj-123', { branch: 'develop' });

      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/commits?branch=develop',
        { method: 'GET' }
      );
    });

    it('should handle empty commit list', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: [],
        error: undefined,
      });

      const result = await getCommitHistory('proj-123');

      expect(result).toEqual([]);
      expect(result).toHaveLength(0);
    });

    it('should throw error when success is false', async () => {
      (apiRequest as any).mockResolvedValue({
        success: false,
        data: undefined,
        error: 'No git repository found',
      });

      await expect(getCommitHistory('proj-123')).rejects.toThrow(
        'No git repository found'
      );
    });

    it('should throw error when data is missing', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: undefined,
        error: undefined,
      });

      await expect(getCommitHistory('proj-123')).rejects.toThrow(
        'Failed to fetch commit history'
      );
    });

    it('should use default error message when no error message provided', async () => {
      (apiRequest as any).mockResolvedValue({
        success: false,
        data: undefined,
        error: undefined,
      });

      await expect(getCommitHistory('proj-123')).rejects.toThrow(
        'Failed to fetch commit history'
      );
    });
  });

  describe('getCommitDetails', () => {
    const mockCommitDetail: CommitDetail = {
      commit: {
        id: 'abc123def456',
        short_id: 'abc123d',
        message: 'Add authentication feature',
        author: 'Test User',
        email: 'test@example.com',
        date: '2025-01-15 12:00:00 UTC',
        timestamp: 1736942400,
        files_changed: 5,
        insertions: 120,
        deletions: 30,
      },
      files: [
        {
          path: 'src/auth.ts',
          old_path: undefined,
          status: 'added',
          insertions: 100,
          deletions: 0,
        },
        {
          path: 'src/config.ts',
          old_path: undefined,
          status: 'modified',
          insertions: 20,
          deletions: 30,
        },
      ],
      parent_ids: ['parent123abc'],
      stats: {
        files_changed: 5,
        total_insertions: 120,
        total_deletions: 30,
      },
    };

    it('should return commit details directly from response.data', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockCommitDetail,
        error: undefined,
      });

      const result = await getCommitDetails('proj-123', 'abc123def456');

      expect(result).toEqual(mockCommitDetail);
      expect(result.commit.id).toBe('abc123def456');
      expect(result.files).toHaveLength(2);
      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/commits/abc123def456',
        { method: 'GET' }
      );
    });

    it('should handle renamed files correctly', async () => {
      const detailWithRename: CommitDetail = {
        ...mockCommitDetail,
        files: [
          {
            path: 'src/new-auth.ts',
            old_path: 'src/old-auth.ts',
            status: 'renamed',
            insertions: 5,
            deletions: 5,
          },
        ],
      };

      (apiRequest as any).mockResolvedValue({
        success: true,
        data: detailWithRename,
        error: undefined,
      });

      const result = await getCommitDetails('proj-123', 'abc123');

      expect(result.files[0].old_path).toBe('src/old-auth.ts');
      expect(result.files[0].status).toBe('renamed');
    });

    it('should throw error when success is false', async () => {
      (apiRequest as any).mockResolvedValue({
        success: false,
        data: undefined,
        error: 'Commit not found',
      });

      await expect(
        getCommitDetails('proj-123', 'invalid-commit')
      ).rejects.toThrow('Commit not found');
    });

    it('should throw error when data is missing', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: undefined,
        error: undefined,
      });

      await expect(
        getCommitDetails('proj-123', 'abc123')
      ).rejects.toThrow('Failed to get commit details');
    });

    it('should use default error message when no error message provided', async () => {
      (apiRequest as any).mockResolvedValue({
        success: false,
        data: undefined,
        error: undefined,
      });

      await expect(
        getCommitDetails('proj-123', 'abc123')
      ).rejects.toThrow('Failed to get commit details');
    });
  });

  describe('getFileDiff', () => {
    const mockFileDiff: FileDiff = {
      path: 'src/auth.ts',
      old_path: undefined,
      status: 'modified',
      content: `@@ -1,3 +1,4 @@
 export function login() {
+  console.log('Login called');
   return authenticate();
 }`,
      is_binary: false,
    };

    it('should return file diff directly from response.data', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockFileDiff,
        error: undefined,
      });

      const result = await getFileDiff(
        'proj-123',
        'abc123',
        'src/auth.ts'
      );

      expect(result).toEqual(mockFileDiff);
      expect(result.path).toBe('src/auth.ts');
      expect(result.is_binary).toBe(false);
      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/diff/abc123/src%2Fauth.ts',
        { method: 'GET' }
      );
    });

    it('should handle context parameter correctly', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockFileDiff,
        error: undefined,
      });

      await getFileDiff('proj-123', 'abc123', 'src/auth.ts', {
        context: 5,
      });

      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/diff/abc123/src%2Fauth.ts?context=5',
        { method: 'GET' }
      );
    });

    it('should handle binary files correctly', async () => {
      const binaryDiff: FileDiff = {
        path: 'assets/image.png',
        old_path: undefined,
        status: 'added',
        content: '',
        is_binary: true,
      };

      (apiRequest as any).mockResolvedValue({
        success: true,
        data: binaryDiff,
        error: undefined,
      });

      const result = await getFileDiff(
        'proj-123',
        'abc123',
        'assets/image.png'
      );

      expect(result.is_binary).toBe(true);
      expect(result.content).toBe('');
    });

    it('should handle file paths with special characters', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: mockFileDiff,
        error: undefined,
      });

      await getFileDiff(
        'proj-123',
        'abc123',
        'src/components/My Component.tsx'
      );

      expect(apiRequest).toHaveBeenCalledWith(
        '/api/git/proj-123/diff/abc123/src%2Fcomponents%2FMy%20Component.tsx',
        { method: 'GET' }
      );
    });

    it('should throw error when success is false', async () => {
      (apiRequest as any).mockResolvedValue({
        success: false,
        data: undefined,
        error: 'File not found in commit',
      });

      await expect(
        getFileDiff('proj-123', 'abc123', 'missing.ts')
      ).rejects.toThrow('File not found in commit');
    });

    it('should throw error when data is missing', async () => {
      (apiRequest as any).mockResolvedValue({
        success: true,
        data: undefined,
        error: undefined,
      });

      await expect(
        getFileDiff('proj-123', 'abc123', 'src/file.ts')
      ).rejects.toThrow('Failed to get file diff');
    });

    it('should use default error message when no error message provided', async () => {
      (apiRequest as any).mockResolvedValue({
        success: false,
        data: undefined,
        error: undefined,
      });

      await expect(
        getFileDiff('proj-123', 'abc123', 'src/file.ts')
      ).rejects.toThrow('Failed to get file diff');
    });
  });

  describe('Utility Functions', () => {
    describe('formatCommitMessage', () => {
      it('should return full message when under max length', () => {
        const message = 'Short commit message';
        expect(formatCommitMessage(message, 72)).toBe(message);
      });

      it('should truncate long messages with ellipsis', () => {
        const message = 'This is a very long commit message that exceeds the maximum length';
        const result = formatCommitMessage(message, 30);
        expect(result).toBe('This is a very long commit ...');
        expect(result.length).toBe(30);
      });

      it('should handle multi-line messages when total message exceeds max', () => {
        const message = 'Line 1\n'.repeat(15);  // Much longer than 72 chars
        const result = formatCommitMessage(message, 72);
        // Returns just the first line since overall message exceeds maxLength
        expect(result).toBe('Line 1');
      });

      it('should truncate long first line in multi-line messages', () => {
        const message = 'This is a very long first line that should be truncated\nSecond line';
        const result = formatCommitMessage(message, 30);
        expect(result).toBe('This is a very long first l...');
        expect(result.length).toBe(30);
      });

      it('should use default max length of 72', () => {
        const message = 'a'.repeat(100);
        const result = formatCommitMessage(message);
        expect(result).toHaveLength(72);
      });
    });

    describe('formatAuthor', () => {
      it('should format author with both name and email', () => {
        const result = formatAuthor('John Doe', 'john@example.com');
        expect(result).toBe('John Doe <john@example.com>');
      });

      it('should return author name when email is missing', () => {
        const result = formatAuthor('John Doe', '');
        expect(result).toBe('John Doe');
      });

      it('should return email when author name is missing', () => {
        const result = formatAuthor('', 'john@example.com');
        expect(result).toBe('john@example.com');
      });

      it('should return "Unknown" when both are missing', () => {
        const result = formatAuthor('', '');
        expect(result).toBe('Unknown');
      });
    });

    describe('formatFileStatus', () => {
      it('should format added status correctly', () => {
        const result = formatFileStatus('added');
        expect(result).toEqual({
          label: 'Added',
          color: 'text-green-600',
          icon: '+',
        });
      });

      it('should format modified status correctly', () => {
        const result = formatFileStatus('modified');
        expect(result).toEqual({
          label: 'Modified',
          color: 'text-blue-600',
          icon: 'M',
        });
      });

      it('should format deleted status correctly', () => {
        const result = formatFileStatus('deleted');
        expect(result).toEqual({
          label: 'Deleted',
          color: 'text-red-600',
          icon: '-',
        });
      });

      it('should format renamed status correctly', () => {
        const result = formatFileStatus('renamed');
        expect(result).toEqual({
          label: 'Renamed',
          color: 'text-purple-600',
          icon: 'R',
        });
      });

      it('should format copied status correctly', () => {
        const result = formatFileStatus('copied');
        expect(result).toEqual({
          label: 'Copied',
          color: 'text-cyan-600',
          icon: 'C',
        });
      });

      it('should handle unknown status', () => {
        const result = formatFileStatus('unknown-status');
        expect(result).toEqual({
          label: 'Unknown',
          color: 'text-gray-600',
          icon: '?',
        });
      });

      it('should be case-insensitive', () => {
        const result = formatFileStatus('MODIFIED');
        expect(result.label).toBe('Modified');
      });
    });
  });
});
