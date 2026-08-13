export function createTaskId(): string {
    return crypto.randomUUID()
}

export function findTaskById(tasks: Task[], id: string): Task | undefined {
    return tasks.find((task) => task.id === id)
}

export function nextPendingTask(tasks: Task[]): Task | undefined {
    return tasks.find((task) => !task.finished && !task.active)
}

export function hasPendingTasks(tasks: Task[]): boolean {
    return tasks.some((task) => !task.finished)
}

export function taskHasWork(task: Task): boolean {
    return Boolean(
        task.url ||
            task.presets.length > 0 ||
            (task.transcript && task.transcript !== 'none') ||
            task.publish
    )
}

export function finishedProgress(tasks: Task[]): number {
    if (tasks.length === 0) return 0
    return Math.round((tasks.filter((task) => task.finished).length * 100) / tasks.length)
}
