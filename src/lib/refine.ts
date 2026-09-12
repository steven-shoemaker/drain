import type { RefinedTask } from "$lib/types";

export type RefineOutcome =
  | { kind: "tasks"; tasks: RefinedTask[] }
  | { kind: "refusal"; question: string };

const ACTION =
  /\b(add|fix|implement|update|remove|create|replace|wire|persist|test|cover|polish|refactor|rename|move|delete|support|allow|show|hide|enable|disable|build|ship|drain|enqueue|capture|refine)\b/i;

const FILLER =
  /^(um+|uh+|idk|dunno|maybe|something|stuff|things|whatever|later|todo|wip|fix it|make it (better|nice|good)|improve (it|this|that)|the thing)\.?$/i;

export function refineMessyIdea(raw: string): RefineOutcome {
  const text = raw.trim().replace(/\s+/g, " ");
  if (!text) {
    return {
      kind: "refusal",
      question: "What should change, and how would we know it worked?",
    };
  }

  const words = text.split(" ").filter(Boolean);
  if (words.length < 6 && !ACTION.test(text)) {
    return {
      kind: "refusal",
      question:
        "Too vague — which surface (screen, file, or API) should change, and how would we know it worked?",
    };
  }
  if (FILLER.test(text) || isWishWithoutTarget(text, words.length)) {
    return {
      kind: "refusal",
      question: "What exactly should a user be able to do when this is done?",
    };
  }

  const chunks = splitIndependentWork(text);
  const tasks = chunks.slice(0, 5).map((chunk, i, all) =>
    taskFromChunk(
      chunk,
      all.filter((_, j) => j !== i),
    ),
  );
  return { kind: "tasks", tasks };
}

function isWishWithoutTarget(text: string, wordCount: number) {
  if (wordCount > 14) return false;
  const hasSurface =
    /\b(inbox|queue|run|settings|sidebar|log|pr|pull request|button|shortcut|repo|sqlite|tauri|git|claude)\b/i.test(
      text,
    );
  const wish = /\b(better|nice|polish|cleanup|clean up|improve|revamp|overhaul)\b/i.test(text);
  return wish && !hasSurface && !ACTION.test(text);
}

export function splitIndependentWork(text: string): string[] {
  const listed = text
    .split(/\n+/)
    .map((l) => l.replace(/^\s*(?:[-*]|\d+[.)])\s+/, "").trim())
    .filter(Boolean);
  if (listed.length >= 2 && listed.every((l) => l.split(" ").length >= 3)) {
    return listed;
  }

  const parts = text
    .split(/\s+(?:and then|and also|plus|as well as|;)\s+/i)
    .map((s) => s.trim())
    .filter(Boolean);

  if (parts.length >= 2 && parts.every((p) => p.split(" ").length >= 4 && ACTION.test(p))) {
    return parts;
  }

  // Two verb phrases joined by "and"
  const andSplit = text.split(/\s+and\s+/i);
  if (
    andSplit.length === 2 &&
    andSplit.every((p) => p.split(" ").length >= 5 && ACTION.test(p))
  ) {
    return andSplit.map((s) => s.trim());
  }

  return [text];
}

function taskFromChunk(chunk: string, siblings: string[]): RefinedTask {
  const title = toImperative(chunk);
  return {
    title,
    acceptanceCriteria: criteriaFrom(chunk),
    outOfScope: [
      ...siblings.map((s) => `Do not also: ${clamp(toImperative(s), 72)}`),
      "Unrelated refactors",
      "New product surfaces",
    ].slice(0, 3),
    suggestedBranch: `drain/${slugify(title)}`,
  };
}

export function toImperative(chunk: string): string {
  const first = chunk.trim().replace(/[.?!]+$/, "");
  if (ACTION.test(first.slice(0, 24))) {
    return clamp(capitalize(first), 80);
  }
  const lowered = first.charAt(0).toLowerCase() + first.slice(1);
  return clamp(`Implement ${lowered}`, 80);
}

function criteriaFrom(chunk: string): string[] {
  const c: string[] = [];
  if (/persist|relaunch|reload|round-?trip/i.test(chunk)) {
    c.push("Value survives a full app relaunch");
  }
  if (/test|cover/i.test(chunk)) {
    c.push("An automated test fails before the change and passes after");
  }
  if (/keyboard|shortcut|hotkey|⌘|cmd/i.test(chunk)) {
    c.push("Shortcut fires when the matching screen is focused");
  }
  if (/empty/i.test(chunk)) {
    c.push("Empty state copy is visible when the list has zero items");
  }
  if (/log|stdout|stderr|stream/i.test(chunk)) {
    c.push("New process output appears in the live log without a reload");
  }
  if (/pr|pull request|approve|merge/i.test(chunk)) {
    c.push("PR URL is stored and Approve / Reject are reachable from Run");
  }
  if (/settings/i.test(chunk)) {
    c.push("The Settings field round-trips after Save");
  }
  if (/inbox|capture/i.test(chunk)) {
    c.push("Capturing an idea shows it in Inbox without a refresh");
  }
  if (/queue|reorder/i.test(chunk)) {
    c.push("Queue order is stable after a reload while idle");
  }
  if (c.length < 2) {
    c.push(`A reviewer can mark this pass/fail: ${clamp(chunk, 90)}`);
    c.push("Existing happy path still works");
  }
  return unique(c).slice(0, 4);
}

function slugify(title: string) {
  return (
    title
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "")
      .slice(0, 40) || "task"
  );
}

function capitalize(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function clamp(s: string, n: number) {
  return s.length <= n ? s : s.slice(0, n - 1).trimEnd();
}

function unique(items: string[]) {
  return [...new Set(items)];
}

export const REFINE_STEPS = [
  { id: "read", label: "Read the messy idea" },
  { id: "split", label: "Split independent work" },
  { id: "criteria", label: "Write checkable acceptance criteria" },
  { id: "branch", label: "Name drain/ branches" },
] as const;
