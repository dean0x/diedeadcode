// Entry point - this is live
import { usedFunction } from './utils';
import { UsedClass } from './classes';

export function main() {
    usedFunction();
    const instance = new UsedClass();
    return instance.method();
}
