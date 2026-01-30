// Live - imported by index.ts
export function usedFunction() {
    return helperFunction();
}

// Live - called by usedFunction
function helperFunction() {
    return 42;
}

// DEAD - never referenced
export function unusedFunction() {
    return 'never called';
}

// DEAD - never referenced
function deadHelper() {
    return 'also never called';
}

// DEAD - type never used
export interface UnusedInterface {
    field: string;
}
