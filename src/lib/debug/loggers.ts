import { isDebugMode } from './config';
import { get } from 'svelte/store';

// Base logger class
class Logger {
    constructor(private name: string) {}

    log(...args: any[]) {
        if (get(isDebugMode)) {
            console.log(`[${this.name}]`, ...args);
        }
    }

    warn(...args: any[]) {
        if (get(isDebugMode)) {
            console.warn(`[${this.name}]`, ...args);
        }
    }

    error(...args: any[]) {
        if (get(isDebugMode)) {
            console.error(`[${this.name}]`, ...args);
        }
    }
}

// Specialized loggers
export const lifecycleLogger = new Logger('Lifecycle');
export const storeLogger = new Logger('Store');
export const apiLogger = new Logger('API');
export const interactionLogger = new Logger('Interaction');

// Reactive logger for Svelte stores
export function watchStore(store: any, name: string) {
    store.subscribe((value: any) => {
        storeLogger.log(`${name} updated:`, value);
    });
}