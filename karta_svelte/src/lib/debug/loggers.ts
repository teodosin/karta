import { isDebugMode } from './config';
import { get } from 'svelte/store';

// Base logger class
function replacer(key: any, value: any) {
    if (value instanceof Map) {
        return {
            dataType: 'Map',
            value: Array.from(value.entries()),
        };
    }
    return value;
}

class Logger {
    private stringifyByDefault: boolean;

    constructor(private name: string, { stringifyByDefault = false } = {}) {
        this.stringifyByDefault = stringifyByDefault;
    }

    private _log(level: 'log' | 'warn' | 'error', args: any[]) {
        if (!get(isDebugMode)) return;

        let localStringify = this.stringifyByDefault;
        let finalArgs = args;

        if (typeof args[0] === 'boolean') {
            localStringify = args[0];
            finalArgs = args.slice(1);
        }

        const processedArgs = localStringify
            ? finalArgs.map(arg => (typeof arg === 'object' ? JSON.stringify(arg, replacer, 2) : arg))
            : finalArgs;

        console[level](`[${this.name}]`, ...processedArgs);
    }

    log(...args: any[]) {
        this._log('log', args);
    }

    warn(...args: any[]) {
        this._log('warn', args);
    }

    error(...args: any[]) {
        this._log('error', args);
    }
}

// Specialized loggers
export const lifecycleLogger = new Logger('Lifecycle');
export const storeLogger = new Logger('Store', { stringifyByDefault: true });
export const apiLogger = new Logger('API');
export const interactionLogger = new Logger('Interaction');

// Reactive logger for Svelte stores
export function watchStore(store: any, name: string) {
    // Temporarily disabled to clean up console
}