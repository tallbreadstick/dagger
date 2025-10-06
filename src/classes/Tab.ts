/**
 * Tab.ts
 * ----------
 * Represents a single tab in the file explorer.
 * Manages directory navigation history and stack traversal.
 */

export default class Tab {
    static serial: number = 0;

    id: number;
    workingDir: string;
    backStack: string[];
    forwardStack: string[];

    constructor(workingDir: string) {
        this.id = Tab.serial++;
        this.workingDir = workingDir;
        this.backStack = [];
        this.forwardStack = [];
    }

    /** Navigate to a new directory */
    navigateTo(newPath: string) {
        if (this.workingDir) {
            this.backStack.push(this.workingDir);
        }
        this.workingDir = newPath;
        this.forwardStack = []; // Clear forward history
    }

    /** Navigate back if possible */
    goBack(): string | null {
        if (this.backStack.length === 0) return null;
        this.forwardStack.push(this.workingDir);
        this.workingDir = this.backStack.pop()!;
        return this.workingDir;
    }

    /** Navigate forward if possible */
    goForward(): string | null {
        if (this.forwardStack.length === 0) return null;
        this.backStack.push(this.workingDir);
        this.workingDir = this.forwardStack.pop()!;
        return this.workingDir;
    }

    /** Go up one directory level */
    goUp(): string | null {
        const parts = this.workingDir.split(/[/\\]+/);
        if (parts.length <= 1) return null; // no parent dir
        parts.pop();
        const parent = parts.join("\\");
        this.navigateTo(parent);
        return parent;
    }

    /** Returns true if there’s a previous directory */
    canGoBack() {
        return this.backStack.length > 0;
    }

    /** Returns true if there’s a forward directory */
    canGoForward() {
        return this.forwardStack.length > 0;
    }

    /** Returns true if there’s a parent directory */
    canGoUp() {
        const parts = this.workingDir.split(/[/\\]+/);
        return parts.length > 1;
    }

    clearHistory() {
        this.backStack = [];
        this.forwardStack = [];
    }

    resetTo(path: string) {
        this.workingDir = path;
        this.clearHistory();
    }

    clone(): Tab {
        const clone = new Tab(this.workingDir);
        clone.backStack = [...this.backStack];
        clone.forwardStack = [...this.forwardStack];
        return clone;
    }
}
