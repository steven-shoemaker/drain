import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { mock } from "$lib/mock";
import type {
  BinaryStatus,
  Idea,
  LogEvent,
  LogLine,
  RefinedTask,
  Settings,
  StatusEvent,
  Task,
} from "$lib/types";

function inTauri() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function isMock() {
  return !inTauri();
}

export async function listIdeas() {
  if (!inTauri()) return mock.listIdeas();
  return invoke<Idea[]>("list_ideas");
}

export async function createIdea(body: string) {
  if (!inTauri()) return mock.createIdea(body);
  return invoke<Idea>("create_idea", { body });
}

export async function getIdea(id: number) {
  if (!inTauri()) return mock.getIdea(id);
  return invoke<Idea>("get_idea", { id });
}

export async function updateIdeaDrafts(id: number, drafts: RefinedTask[]) {
  if (!inTauri()) return mock.updateIdeaDrafts(id, drafts);
  return invoke<Idea>("update_idea_drafts", { id, drafts });
}

export async function updateIdeaBody(id: number, body: string) {
  if (!inTauri()) return mock.updateIdeaBody(id, body);
  return invoke<Idea>("update_idea_body", { id, body });
}

export async function refineIdea(id: number) {
  if (!inTauri()) return mock.refineIdea(id);
  return invoke<void>("refine_idea", { id });
}

export async function enqueueIdea(id: number, drafts: RefinedTask[], repoPath?: string) {
  if (!inTauri()) return mock.enqueueIdea(id, drafts, repoPath);
  return invoke<Task[]>("enqueue_idea", { id, drafts, repoPath });
}

export async function listTasks() {
  if (!inTauri()) return mock.listTasks();
  return invoke<Task[]>("list_tasks");
}

export async function getTask(id: number) {
  if (!inTauri()) return mock.getTask(id);
  return invoke<Task>("get_task", { id });
}

export async function reorderTasks(ids: number[]) {
  if (!inTauri()) return mock.reorderTasks(ids);
  return invoke<Task[]>("reorder_tasks", { ids });
}

export async function startQueue() {
  if (!inTauri()) return mock.startQueue();
  return invoke<void>("start_queue");
}

export async function pauseQueue() {
  if (!inTauri()) return mock.pauseQueue();
  return invoke<void>("pause_queue");
}

export async function queuePaused() {
  if (!inTauri()) return mock.queuePaused();
  return invoke<boolean>("queue_paused");
}

export async function cancelRun() {
  if (!inTauri()) return mock.cancelRun();
  return invoke<void>("cancel_run");
}

export async function approveTask(id: number) {
  if (!inTauri()) return mock.approveTask(id);
  return invoke<Task>("approve_task", { id });
}

export async function rejectTask(id: number, note: string) {
  if (!inTauri()) return mock.rejectTask(id, note);
  return invoke<Task>("reject_task", { id, note });
}

export async function retryTask(id: number) {
  if (!inTauri()) return mock.retryTask(id);
  return invoke<Task>("retry_task", { id });
}

export async function skipFailed(id: number) {
  if (!inTauri()) return mock.skipFailed();
  return invoke<void>("skip_failed", { id });
}

export async function getSettings() {
  if (!inTauri()) return mock.getSettings();
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings) {
  if (!inTauri()) return mock.saveSettings(settings);
  return invoke<Settings>("save_settings", { settings });
}

export async function getRunLog(taskId: number) {
  if (!inTauri()) return mock.getRunLog(taskId);
  return invoke<LogLine[]>("get_run_log", { taskId });
}

export async function binaryStatus() {
  if (!inTauri()) return mock.binaryStatus();
  return invoke<BinaryStatus>("binary_status");
}

export async function pickFolder() {
  if (!inTauri()) return mock.pickFolder();
  return invoke<string | null>("pick_folder");
}

export async function openUrl(url: string) {
  if (!inTauri()) return mock.openUrl(url);
  return invoke<void>("open_url", { url });
}

export async function listenLog(fn: (e: LogEvent) => void): Promise<UnlistenFn> {
  if (!inTauri()) return mock.onLog(fn);
  return listen<LogEvent>("drain://log", (e) => fn(e.payload));
}

export async function listenStatus(fn: (e: StatusEvent) => void): Promise<UnlistenFn> {
  if (!inTauri()) return mock.onStatus(fn);
  return listen<StatusEvent>("drain://status", (e) => fn(e.payload));
}
