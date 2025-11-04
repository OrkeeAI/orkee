// ABOUTME: Main IPC bridge entry point for Vibekit SDK integration
// ABOUTME: Handles stdin/stdout communication with Rust parent process

import * as readline from 'readline';
import { VibekitManager } from './vibekit';
import type {
  IPCRequest,
  IPCResponse,
  ExecutionRequest,
  ExecutionResponse,
  LogEntry,
  Artifact,
  ResourceUsage,
  ExecutionStatus,
} from './types';

class VibekitBridge {
  private manager: VibekitManager;
  private rl: readline.Interface;
  private isShuttingDown = false;

  constructor() {
    // Set up readline for stdin
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false,
    });

    // Create Vibekit manager with event handlers
    this.manager = new VibekitManager({
      onExecutionStarted: this.handleExecutionStarted.bind(this),
      onLog: this.handleLog.bind(this),
      onArtifact: this.handleArtifact.bind(this),
      onResourceUpdate: this.handleResourceUpdate.bind(this),
      onComplete: this.handleComplete.bind(this),
      onError: this.handleError.bind(this),
    });
  }

  async start(): Promise<void> {
    try {
      // Initialize Vibekit SDK
      await this.manager.initialize();
      this.sendResponse({ type: 'pong' }); // Signal ready to parent

      // Set up message handling
      this.rl.on('line', (line) => {
        this.handleMessage(line).catch((error) => {
          this.sendResponse({
            type: 'error',
            error: 'Failed to handle message',
            details: error instanceof Error ? error.message : String(error),
          });
        });
      });

      // Handle process termination
      this.setupShutdownHandlers();
    } catch (error) {
      this.sendResponse({
        type: 'error',
        error: 'Failed to initialize Vibekit bridge',
        details: error instanceof Error ? error.message : String(error),
      });
      process.exit(1);
    }
  }

  private async handleMessage(line: string): Promise<void> {
    if (this.isShuttingDown) {
      return;
    }

    try {
      const message: IPCRequest = JSON.parse(line);

      switch (message.type) {
        case 'ping':
          this.sendResponse({ type: 'pong' });
          break;

        case 'execute':
          await this.handleExecute(message.request);
          break;

        case 'stop':
          await this.handleStop(message.execution_id);
          break;

        default:
          this.sendResponse({
            type: 'error',
            error: 'Unknown message type',
            details: JSON.stringify(message),
          });
      }
    } catch (error) {
      this.sendResponse({
        type: 'error',
        error: 'Failed to parse message',
        details: error instanceof Error ? error.message : String(error),
      });
    }
  }

  private async handleExecute(request: ExecutionRequest): Promise<void> {
    try {
      await this.manager.executeTask(request);
      // Response is sent via onExecutionStarted handler
    } catch (error) {
      this.sendResponse({
        type: 'error',
        execution_id: request.execution_id,
        error: 'Failed to execute task',
        details: error instanceof Error ? error.message : String(error),
      });
    }
  }

  private async handleStop(executionId: string): Promise<void> {
    try {
      await this.manager.stopExecution(executionId);
    } catch (error) {
      this.sendResponse({
        type: 'error',
        execution_id: executionId,
        error: 'Failed to stop execution',
        details: error instanceof Error ? error.message : String(error),
      });
    }
  }

  private handleExecutionStarted(response: ExecutionResponse): void {
    this.sendResponse({
      type: 'execution_started',
      response,
    });
  }

  private handleLog(log: LogEntry): void {
    this.sendResponse({
      type: 'log',
      log,
    });
  }

  private handleArtifact(artifact: Artifact): void {
    this.sendResponse({
      type: 'artifact',
      artifact,
    });
  }

  private handleResourceUpdate(executionId: string, usage: ResourceUsage): void {
    this.sendResponse({
      type: 'resource_update',
      execution_id: executionId,
      usage,
    });
  }

  private handleComplete(
    executionId: string,
    status: ExecutionStatus,
    errorMessage?: string
  ): void {
    this.sendResponse({
      type: 'execution_complete',
      execution_id: executionId,
      status,
      error_message: errorMessage,
    });
  }

  private handleError(executionId: string, error: string, details?: string): void {
    this.sendResponse({
      type: 'error',
      execution_id: executionId,
      error,
      details,
    });
  }

  private sendResponse(response: IPCResponse): void {
    if (this.isShuttingDown) {
      return;
    }

    try {
      const message = JSON.stringify(response);
      console.log(message); // stdout to parent process
    } catch (error) {
      // Can't send error via sendResponse, log to stderr
      console.error('Failed to serialize response:', error);
    }
  }

  private setupShutdownHandlers(): void {
    const shutdown = async (signal: string) => {
      if (this.isShuttingDown) {
        return;
      }

      this.isShuttingDown = true;
      console.error(`Received ${signal}, shutting down gracefully...`);

      try {
        await this.manager.cleanup();
        this.rl.close();
        process.exit(0);
      } catch (error) {
        console.error('Error during shutdown:', error);
        process.exit(1);
      }
    };

    process.on('SIGINT', () => shutdown('SIGINT'));
    process.on('SIGTERM', () => shutdown('SIGTERM'));

    // Handle parent process exit
    process.stdin.on('end', () => {
      console.error('Parent process closed stdin, shutting down...');
      shutdown('STDIN_END').catch(() => process.exit(1));
    });

    // Handle uncaught errors
    process.on('uncaughtException', (error) => {
      console.error('Uncaught exception:', error);
      this.sendResponse({
        type: 'error',
        error: 'Uncaught exception',
        details: error.message,
      });
      shutdown('UNCAUGHT_EXCEPTION').catch(() => process.exit(1));
    });

    process.on('unhandledRejection', (reason) => {
      console.error('Unhandled rejection:', reason);
      this.sendResponse({
        type: 'error',
        error: 'Unhandled rejection',
        details: String(reason),
      });
    });
  }
}

// Start the bridge
const bridge = new VibekitBridge();
bridge.start().catch((error) => {
  console.error('Failed to start Vibekit bridge:', error);
  process.exit(1);
});
