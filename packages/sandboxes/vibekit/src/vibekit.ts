// ABOUTME: Vibekit SDK integration and session management
// ABOUTME: Handles agent execution, log streaming, and resource monitoring via Vibekit

import type {
  ExecutionRequest,
  ExecutionResponse,
  ExecutionStatus,
  LogEntry,
  Artifact,
  ResourceUsage,
} from './types';

// Vibekit SDK types (will be provided by @vibe-kit/sdk)
interface VibekitSession {
  id: string;
  execute(prompt: string): Promise<void>;
  stop(): Promise<void>;
  on(event: string, handler: (...args: any[]) => void): void;
  off(event: string, handler: (...args: any[]) => void): void;
}

interface VibekitClient {
  createSession(config: SessionConfig): Promise<VibekitSession>;
  destroySession(sessionId: string): Promise<void>;
}

interface SessionConfig {
  provider: string;
  containerImage: string;
  agentId: string;
  model: string;
  resourceLimits: {
    memoryMB: number;
    cpuCores: number;
    timeoutSeconds: number;
  };
  workspacePath?: string;
  environmentVariables?: Record<string, string>;
}

// Event handlers for execution lifecycle
export interface ExecutionEventHandlers {
  onExecutionStarted: (response: ExecutionResponse) => void;
  onLog: (log: LogEntry) => void;
  onArtifact: (artifact: Artifact) => void;
  onResourceUpdate: (executionId: string, usage: ResourceUsage) => void;
  onComplete: (executionId: string, status: ExecutionStatus, error?: string) => void;
  onError: (executionId: string, error: string, details?: string) => void;
}

// Active execution tracking
interface ActiveExecution {
  request: ExecutionRequest;
  session: VibekitSession;
  logSequence: number;
  startTime: Date;
}

export class VibekitManager {
  private client: VibekitClient | null = null;
  private activeExecutions: Map<string, ActiveExecution> = new Map();
  private handlers: ExecutionEventHandlers;

  constructor(handlers: ExecutionEventHandlers) {
    this.handlers = handlers;
  }

  async initialize(): Promise<void> {
    try {
      // Import Vibekit SDK dynamically
      const { VibekitClient } = await import('@vibe-kit/sdk');
      this.client = new VibekitClient();
    } catch (error) {
      throw new Error(
        `Failed to initialize Vibekit SDK: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  async executeTask(request: ExecutionRequest): Promise<ExecutionResponse> {
    if (!this.client) {
      throw new Error('Vibekit client not initialized');
    }

    if (this.activeExecutions.has(request.execution_id)) {
      throw new Error(`Execution ${request.execution_id} is already running`);
    }

    try {
      // Create Vibekit session
      const sessionConfig: SessionConfig = {
        provider: request.provider,
        containerImage: request.container_image,
        agentId: request.agent_id,
        model: request.model,
        resourceLimits: {
          memoryMB: request.resource_limits.memory_mb,
          cpuCores: request.resource_limits.cpu_cores,
          timeoutSeconds: request.resource_limits.timeout_seconds,
        },
        workspacePath: request.workspace_path,
        environmentVariables: request.environment_variables,
      };

      const session = await this.client.createSession(sessionConfig);

      // Track execution
      const execution: ActiveExecution = {
        request,
        session,
        logSequence: 0,
        startTime: new Date(),
      };
      this.activeExecutions.set(request.execution_id, execution);

      // Set up event handlers
      this.setupSessionHandlers(request.execution_id, session);

      // Start execution
      await session.execute(request.prompt);

      // Return initial response
      const response: ExecutionResponse = {
        execution_id: request.execution_id,
        container_id: session.id,
        status: 'running',
        container_status: 'running',
        vibekit_session_id: session.id,
      };

      this.handlers.onExecutionStarted(response);

      return response;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.handlers.onError(request.execution_id, 'Failed to start execution', errorMessage);

      return {
        execution_id: request.execution_id,
        container_id: '',
        status: 'failed',
        container_status: 'error',
        error_message: errorMessage,
      };
    }
  }

  async stopExecution(executionId: string): Promise<void> {
    const execution = this.activeExecutions.get(executionId);
    if (!execution) {
      throw new Error(`Execution ${executionId} not found`);
    }

    try {
      await execution.session.stop();
      this.activeExecutions.delete(executionId);
      this.handlers.onComplete(executionId, 'cancelled');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.handlers.onError(executionId, 'Failed to stop execution', errorMessage);
    }
  }

  private setupSessionHandlers(executionId: string, session: VibekitSession): void {
    const execution = this.activeExecutions.get(executionId);
    if (!execution) return;

    // Handle log messages
    session.on('log', (level: string, message: string, metadata?: Record<string, unknown>) => {
      const log: LogEntry = {
        id: this.generateLogId(),
        execution_id: executionId,
        timestamp: new Date().toISOString(),
        log_level: level as LogEntry['log_level'],
        message,
        source: 'vibekit',
        metadata,
        sequence_number: execution.logSequence++,
      };
      this.handlers.onLog(log);
    });

    // Handle errors
    session.on('error', (error: Error, stackTrace?: string) => {
      const log: LogEntry = {
        id: this.generateLogId(),
        execution_id: executionId,
        timestamp: new Date().toISOString(),
        log_level: 'error',
        message: error.message,
        source: 'vibekit',
        stack_trace: stackTrace || error.stack,
        sequence_number: execution.logSequence++,
      };
      this.handlers.onLog(log);
      this.handlers.onError(executionId, error.message, stackTrace || error.stack);
    });

    // Handle completion
    session.on('complete', (status: string, error?: string) => {
      this.activeExecutions.delete(executionId);
      this.handlers.onComplete(
        executionId,
        status as ExecutionStatus,
        error
      );
    });

    // Handle resource updates
    session.on('resources', (memoryMB: number, cpuPercent: number) => {
      const usage: ResourceUsage = {
        memory_used_mb: memoryMB,
        cpu_usage_percent: cpuPercent,
      };
      this.handlers.onResourceUpdate(executionId, usage);
    });

    // Handle artifacts
    session.on('artifact', (artifactData: any) => {
      const artifact: Artifact = {
        id: this.generateArtifactId(),
        execution_id: executionId,
        artifact_type: artifactData.type || 'output',
        file_path: artifactData.path,
        file_name: artifactData.name,
        file_size_bytes: artifactData.size,
        mime_type: artifactData.mimeType,
        storage_backend: 'local',
        created_at: new Date().toISOString(),
      };
      this.handlers.onArtifact(artifact);
    });
  }

  async cleanup(): Promise<void> {
    // Stop all active executions
    const executionIds = Array.from(this.activeExecutions.keys());
    await Promise.all(
      executionIds.map((id) => this.stopExecution(id).catch(() => {
        // Ignore errors during cleanup
      }))
    );
  }

  private generateLogId(): string {
    return `log_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  private generateArtifactId(): string {
    return `art_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }
}
