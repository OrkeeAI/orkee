import { useQuery } from '@tanstack/react-query';
import { apiRequest } from './api';

// Type definitions
export interface CommitInfo {
  id: string;
  short_id: string;
  message: string;
  author: string;
  email: string;
  date: string;
  timestamp: number;
  files_changed: number;
  insertions: number;
  deletions: number;
}

export interface CommitDetail {
  commit: CommitInfo;
  files: FileChange[];
  parent_ids: string[];
  stats: CommitStats;
}

export interface FileChange {
  path: string;
  old_path?: string;
  status: string;
  insertions: number;
  deletions: number;
}

export interface CommitStats {
  files_changed: number;
  total_insertions: number;
  total_deletions: number;
}

export interface FileDiff {
  path: string;
  old_path?: string;
  status: string;
  content: string;
  is_binary: boolean;
}

export interface CommitHistoryParams {
  page?: number;
  per_page?: number;
  branch?: string;
}

export interface FileDiffParams {
  context?: number;
}

// API functions
export const getCommitHistory = async (
  projectId: string,
  params: CommitHistoryParams = {}
): Promise<CommitInfo[]> => {
  const searchParams = new URLSearchParams();

  if (params.page) {
    searchParams.append('page', params.page.toString());
  }
  if (params.per_page) {
    searchParams.append('per_page', params.per_page.toString());
  }
  if (params.branch) {
    searchParams.append('branch', params.branch);
  }

  const queryString = searchParams.toString();
  const url = `/api/git/${projectId}/commits${queryString ? '?' + queryString : ''}`;

  const response = await apiRequest<{ success: boolean; data?: CommitInfo[]; error?: string }>(url, {
    method: 'GET',
  });

  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to fetch commit history');
  }

  return response.data;
};

export const getCommitDetails = async (
  projectId: string,
  commitId: string
): Promise<CommitDetail> => {
  const response = await apiRequest<{ success: boolean; data?: CommitDetail; error?: string }>(
    `/api/git/${projectId}/commits/${commitId}`,
    {
      method: 'GET',
    }
  );

  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get commit details');
  }

  return response.data;
};

export const getFileDiff = async (
  projectId: string,
  commitId: string,
  filePath: string,
  params: FileDiffParams = {}
): Promise<FileDiff> => {
  const searchParams = new URLSearchParams();

  if (params.context !== undefined) {
    searchParams.append('context', params.context.toString());
  }

  const queryString = searchParams.toString();
  const url = `/api/git/${projectId}/diff/${commitId}/${encodeURIComponent(filePath)}${
    queryString ? '?' + queryString : ''
  }`;

  const response = await apiRequest<{ success: boolean; data?: FileDiff; error?: string }>(url, {
    method: 'GET',
  });

  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get file diff');
  }

  return response.data;
};

// React Query hooks
export const useCommitHistory = (
  projectId: string,
  params: CommitHistoryParams = {},
  options: { enabled?: boolean } = {}
) => {
  return useQuery({
    queryKey: ['commitHistory', projectId, params],
    queryFn: () => getCommitHistory(projectId, params),
    staleTime: 5 * 60 * 1000, // 5 minutes
    enabled: options.enabled !== false && !!projectId,
  });
};

export const useCommitDetails = (
  projectId: string,
  commitId: string,
  options: { enabled?: boolean } = {}
) => {
  return useQuery({
    queryKey: ['commitDetails', projectId, commitId],
    queryFn: () => getCommitDetails(projectId, commitId),
    staleTime: 10 * 60 * 1000, // 10 minutes
    enabled: options.enabled !== false && !!projectId && !!commitId,
  });
};

export const useFileDiff = (
  projectId: string,
  commitId: string,
  filePath: string,
  params: FileDiffParams = {},
  options: { enabled?: boolean } = {}
) => {
  return useQuery({
    queryKey: ['fileDiff', projectId, commitId, filePath, params],
    queryFn: () => getFileDiff(projectId, commitId, filePath, params),
    staleTime: 10 * 60 * 1000, // 10 minutes
    enabled: options.enabled !== false && !!projectId && !!commitId && !!filePath,
  });
};

// Utility functions
export const formatCommitMessage = (message: string, maxLength: number = 72): string => {
  if (message.length <= maxLength) {
    return message;
  }
  
  const lines = message.split('\n');
  const firstLine = lines[0];
  
  if (firstLine.length <= maxLength) {
    return firstLine;
  }
  
  return firstLine.substring(0, maxLength - 3) + '...';
};

export const formatAuthor = (author: string, email: string): string => {
  if (!author && !email) {
    return 'Unknown';
  }
  
  if (!email) {
    return author;
  }
  
  if (!author) {
    return email;
  }
  
  return `${author} <${email}>`;
};

export const formatFileStatus = (status: string): { 
  label: string; 
  color: string; 
  icon: string;
} => {
  switch (status.toLowerCase()) {
    case 'added':
      return { label: 'Added', color: 'text-green-600', icon: '+' };
    case 'modified':
      return { label: 'Modified', color: 'text-blue-600', icon: 'M' };
    case 'deleted':
      return { label: 'Deleted', color: 'text-red-600', icon: '-' };
    case 'renamed':
      return { label: 'Renamed', color: 'text-purple-600', icon: 'R' };
    case 'copied':
      return { label: 'Copied', color: 'text-cyan-600', icon: 'C' };
    default:
      return { label: 'Unknown', color: 'text-gray-600', icon: '?' };
  }
};

export const getFileExtension = (filePath: string): string => {
  const parts = filePath.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
};

export const getLanguageFromExtension = (extension: string): string => {
  const languageMap: Record<string, string> = {
    // Web technologies
    'js': 'javascript',
    'jsx': 'javascript',
    'ts': 'typescript',
    'tsx': 'typescript',
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'less': 'less',
    'vue': 'vue',
    'json': 'json',
    
    // Programming languages
    'py': 'python',
    'rb': 'ruby',
    'php': 'php',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c',
    'cs': 'csharp',
    'go': 'go',
    'rs': 'rust',
    'swift': 'swift',
    'kt': 'kotlin',
    'scala': 'scala',
    
    // Markup and config
    'md': 'markdown',
    'xml': 'xml',
    'yaml': 'yaml',
    'yml': 'yaml',
    'toml': 'toml',
    'ini': 'ini',
    'conf': 'ini',
    
    // Shell and scripts
    'sh': 'bash',
    'bash': 'bash',
    'zsh': 'bash',
    'fish': 'bash',
    'ps1': 'powershell',
    
    // Database
    'sql': 'sql',
    
    // Docker
    'dockerfile': 'docker',
  };
  
  return languageMap[extension] || 'text';
};