// Live - imported by index.ts
export class UsedClass {
    method() {
        return 'used';
    }

    // DEAD - method never called
    unusedMethod() {
        return 'never called';
    }
}

// DEAD - class never instantiated
export class UnusedClass {
    value: number = 0;

    compute() {
        return this.value * 2;
    }
}

// DEAD - type never referenced
export type UnusedType = {
    id: number;
    name: string;
};
