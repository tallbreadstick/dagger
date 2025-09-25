export default class Tab {

    static serial: number = 0;

    id: number;
    stack: string[];
    workingDir: string;

    constructor(workingDir: string) {
        this.id = Tab.serial++;
        this.workingDir = workingDir;
        this.stack = [];
    }

    isEmpty() {
        return this.stack.length === 0;
    }

    top() {
        return this.stack[this.stack.length - 1];
    }

    push(path: string) {
        this.stack.push(path);
    }

    pop() {
        return this.stack.pop();
    }

    clear() {
        this.stack = [];
    }

}