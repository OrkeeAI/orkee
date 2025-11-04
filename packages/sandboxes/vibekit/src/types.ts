// ABOUTME: TypeScript type definitions for Vibekit bridge IPC communication
// ABOUTME: Mirrors Rust types and defines IPC message protocol between Rust and Node.js

export type SandboxProvider = 'local' | 'e2b' | 'modal';

export type ContainerStatus = 'creating' | 'running' | 'stopped' | 'error';

export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface ResourceLimits {
  memory_mb: number;
  cpu_cores: number;
  timeout_seconds: number;
}

export interface ExecutionRequest {
  execution_id: string;
  task_id: string;
  agent_id: string;
  model: string;
  prompt: string;
  provider: SandboxProvider;
  container_image: string;
  resource_limits: ResourceLimits;
  workspace_path?: string;
  environment_variables: Record<string, string>;
}

export interface ExecutionResponse {
  execution_id: string;
  container_id: string;
  status: ExecutionStatus;
  container_status: ContainerStatus;
  vibekit_session_id?: string;
  error_message?: string;
}

export interface LogEntry {
  id: string;
  execution_id: string;
  timestamp: string;
  log_level: 'debug' | 'info' | 'warn' | 'error' | 'fatal';
  message: string;
  source?: 'vibekit' | 'agent' | 'container' | 'system';
  metadata?: Record<string, unknown>;
  stack_trace?: string;
  sequence_number: number;
}

export interface Artifact {
  id: string;
  execution_id: string;
  artifact_type: 'file' | 'screenshot' | 'test_report' | 'coverage' | 'output';
  file_path: string;
  file_name: string;
  file_size_bytes?: number;
  mime_type?: string;
  stored_path?: string;
  storage_backend: 'local' | 's3' | 'gcs';
  description?: string;
  metadata?: Record<string, unknown>;
  checksum?: string;
  created_at: string;
}

export interface ResourceUsage {
  memory_used_mb: number;
  cpu_usage_percent: number;
}

// IPC Message Types for communication between Rust and Node.js

export interface IPCExecuteRequest {
  type: 'execute';
  request: ExecutionRequest;
}

export interface IPCStopRequest {
  type: 'stop';
  execution_id: string;
}

export interface IPCPingRequest {
  type: 'ping';
}

export type IPCRequest = IPCExecuteRequest | IPCStopRequest | IPCPingRequest;

export interface IPCExecutionStarted {
  type: 'execution_started';
  response: ExecutionResponse;
}

export interface IPCLogMessage {
  type: 'log';
  log: LogEntry;
}

export interface IPCExecutionComplete {
  type: 'execution_complete';
  execution_id: string;
  status: ExecutionStatus;
  error_message?: string;
}

export interface IPCArtifactCreated {
  type: 'artifact';
  artifact: Artifact;
}

export interface IPCResourceUpdate {
  type: 'resource_update';
  execution_id: string;
  usage: ResourceUsage;
}

export interface IPCError {
  type: 'error';
  execution_id?: string;
  error: string;
  details?: string;
}

export interface IPCPong {
  type: 'pong';
}

export type IPCResponse =
  | IPCExecutionStarted
  | IPCLogMessage
  | IPCExecutionComplete
  | IPCArtifactCreated
  | IPCResourceUpdate
  | IPCError
  | IPCPong;
