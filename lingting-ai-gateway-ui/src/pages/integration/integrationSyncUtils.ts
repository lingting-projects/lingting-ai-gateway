import type { IntegrationSyncTarget } from "./integrationAgents";

/** 同步目标缓存键前缀；每个 agent 一个键。 */
const STORAGE_KEY_PREFIX = "lingting-ai-gateway:integration-sync";

/** 同步目标的本地缓存。 */
export type IntegrationSyncCache = {
  /** 历史手动填写过的目标 */
  history: IntegrationSyncTarget[];
  /** 上次使用的目标 */
  last?: IntegrationSyncTarget | null;
};

function storageKey(agentKey: string): string {
  return `${STORAGE_KEY_PREFIX}:${agentKey}`;
}

function emptyCache(): IntegrationSyncCache {
  return { history: [], last: null };
}

/** 解析缓存中的单个目标：结构或字段非法时返回 null。 */
function readTarget(value: unknown): IntegrationSyncTarget | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const { home, name } = value as Partial<IntegrationSyncTarget>;
  if (typeof home !== "string" || typeof name !== "string" || !home.trim()) {
    return null;
  }
  return { home, name };
}

/** 按 home 去重，保留先出现的项。 */
function uniqueTargets(targets: IntegrationSyncTarget[]): IntegrationSyncTarget[] {
  const seen = new Set<string>();
  return targets.filter((target) => {
    if (seen.has(target.home)) {
      return false;
    }
    seen.add(target.home);
    return true;
  });
}

/** 读取本地缓存：缺失、损坏或非法时回落到空缓存。 */
export function readSyncCache(agentKey: string): IntegrationSyncCache {
  try {
    const raw = window.localStorage.getItem(storageKey(agentKey));
    if (!raw) {
      return emptyCache();
    }

    const stored = JSON.parse(raw) as Partial<IntegrationSyncCache>;
    const history = Array.isArray(stored.history)
      ? stored.history.map(readTarget).filter((target) => target !== null)
      : [];
    return { history, last: readTarget(stored.last) };
  } catch {
    return emptyCache();
  }
}

/** 写入本地缓存：本地存储不可用时忽略。 */
function writeSyncCache(agentKey: string, cache: IntegrationSyncCache): void {
  try {
    window.localStorage.setItem(storageKey(agentKey), JSON.stringify(cache));
  } catch {
    // 本地存储不可用时仅保留内存状态
  }
}

/**
 * 合并可选目标：服务端数据 → 本地历史 → 上次使用，按 home 去重。
 *
 * 服务端数据排在最前，因此同一个文件夹以服务端给出的名称优先。
 */
export function buildSyncTargets(
  serverTargets: IntegrationSyncTarget[],
  cache: IntegrationSyncCache,
): IntegrationSyncTarget[] {
  return uniqueTargets([
    ...serverTargets,
    ...cache.history,
    ...(cache.last ? [cache.last] : []),
  ]);
}

/** 记录本次同步目标并返回更新后的缓存：更新「上次使用」，非服务端来源的目录并入历史。 */
export function recordSyncTarget(
  agentKey: string,
  target: IntegrationSyncTarget,
  serverTargets: IntegrationSyncTarget[],
): IntegrationSyncCache {
  const cache = readSyncCache(agentKey);
  const fromServer = serverTargets.some((item) => item.home === target.home);
  const others = cache.history.filter((item) => item.home !== target.home);
  const next: IntegrationSyncCache = {
    history: fromServer ? others : [target, ...others],
    last: target,
  };

  writeSyncCache(agentKey, next);
  return next;
}
