import { TaskProvider, TaskProviderType, TaskProviderConfig } from '../types';
import { ManualTaskProvider } from './manual';

export class TaskProviderFactory {
  private static providers = new Map<TaskProviderType, any>([
    [TaskProviderType.Manual, ManualTaskProvider],
  ]);

  static create(config: TaskProviderConfig): TaskProvider {
    const ProviderClass = this.providers.get(config.type);

    if (!ProviderClass) {
      throw new Error(`Unknown task provider type: ${config.type}`);
    }

    const provider = new ProviderClass(config.options);
    return provider;
  }

  static register(type: TaskProviderType, providerClass: new (options?: any) => TaskProvider): void {
    this.providers.set(type, providerClass);
  }

  static getAvailableProviders(): TaskProviderType[] {
    return Array.from(this.providers.keys());
  }
}