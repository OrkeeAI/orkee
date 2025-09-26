import { TaskProvider, TaskProviderType, TaskProviderConfig } from '../types';
import { TaskmasterProvider } from './taskmaster';

export class TaskProviderFactory {
  private static providers = new Map<TaskProviderType, any>([
    [TaskProviderType.Taskmaster, TaskmasterProvider],
  ]);

  static create(config: TaskProviderConfig): TaskProvider {
    const ProviderClass = this.providers.get(config.type);
    
    if (!ProviderClass) {
      throw new Error(`Unknown task provider type: ${config.type}`);
    }
    
    // Create provider with options if it's TaskmasterProvider
    if (config.type === TaskProviderType.Taskmaster && config.options?.apiBaseUrl) {
      return new ProviderClass(config.options.apiBaseUrl);
    }
    
    const provider = new ProviderClass();
    return provider;
  }
  
  static register(type: TaskProviderType, providerClass: new () => TaskProvider): void {
    this.providers.set(type, providerClass);
  }
  
  static getAvailableProviders(): TaskProviderType[] {
    return Array.from(this.providers.keys());
  }
}