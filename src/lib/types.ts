export type Status =
  | "idea"
  | "refining"
  | "ready"
  | "running"
  | "awaiting_pr"
  | "awaiting_approve"
  | "done"
  | "failed";

export type CompletionMode = "pr_then_approve" | "push_and_merge";
export type ApproveAction = "merge" | "open_browser";
export type PermissionMode = "bypassPermissions" | "acceptEdits" | "default";

export type RefinedTask = {
  title: string;
  acceptanceCriteria: string[];
  outOfScope: string[];
  suggestedBranch: string;
};

export type Idea = {
  id: number;
  body: string;
  status: Status;
  error: string | null;
  drafts: RefinedTask[];
  createdAt: string;
  updatedAt: string;
};

export type Task = {
  id: number;
  ideaId: number | null;
  title: string;
  acceptanceCriteria: string[];
  outOfScope: string[];
  suggestedBranch: string;
  repoPath: string;
  status: Status;
  position: number;
  prUrl: string | null;
  failNote: string | null;
  createdAt: string;
  updatedAt: string;
};

export type Settings = {
  defaultRepoPath: string;
  pathToClaude: string;
  pathToGh: string;
  autoRun: boolean;
  completionMode: CompletionMode;
  approveAction: ApproveAction;
  permissionMode: PermissionMode;
};

export type LogLine = {
  id: number;
  taskId: number;
  stream: "stdout" | "stderr" | "system" | string;
  line: string;
  ts: string;
};

export type LogEvent = {
  taskId: number;
  stream: string;
  line: string;
};

export type StatusEvent = {
  taskId: number | null;
  status: string;
  message: string | null;
};

export type BinaryStatus = {
  git: string | null;
  claude: string | null;
  gh: string | null;
};

export type ThinkStep = {
  id: string;
  label: string;
  state: "pending" | "running" | "done";
};

export type RefineLogEvent = {
  ideaId: number;
  stream: string;
  line: string;
};

export const STATUS_LABEL: Record<Status, string> = {
  idea: "idea",
  refining: "refining",
  ready: "ready",
  running: "running",
  awaiting_pr: "opening pr",
  awaiting_approve: "awaiting approve",
  done: "done",
  failed: "failed",
};
