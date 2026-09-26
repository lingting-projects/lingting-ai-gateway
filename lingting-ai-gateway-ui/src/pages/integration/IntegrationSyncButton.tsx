import { SyncOutlined } from "@ant-design/icons";
import { AppHolder, Button } from "@lri";
import { useCallback, useMemo, useState } from "react";

import type { IntegrationAgent, IntegrationSyncTarget } from "./integrationAgents";
import { IntegrationSyncModal } from "./IntegrationSyncModal";
import { buildSyncTargets, readSyncCache, recordSyncTarget } from "./integrationSyncUtils";

type IntegrationSyncButtonProps = {
  agent: IntegrationAgent;
  /** 服务端提供的同步目标 */
  serverTargets: IntegrationSyncTarget[];
};

/**
 * 集成同步入口：开发模式直接同步，其余模式先弹窗确认同步目标。
 */
export function IntegrationSyncButton({ agent, serverTargets }: IntegrationSyncButtonProps) {
  const [open, setOpen] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [cache, setCache] = useState(() => readSyncCache(agent.key));

  const targets = useMemo(() => buildSyncTargets(serverTargets, cache), [cache, serverTargets]);

  /** 执行同步：带目标时写入该目录并记入本地缓存，不带目标时由后端取默认目录。 */
  const runSync = useCallback(
    async (target?: IntegrationSyncTarget) => {
      try {
        const backup = await agent.sync(target?.home);
        if (target) {
          setCache(recordSyncTarget(agent.key, target, serverTargets));
        }
        void AppHolder.message.success(backup ? `同步成功，原配置已备份至 ${backup}` : "同步成功");
        return true;
      } catch (error) {
        void AppHolder.message.error(error instanceof Error ? error.message : "同步失败");
        return false;
      }
    },
    [agent, serverTargets],
  );

  const handleClick = useCallback(() => {
    if (import.meta.env.MODE !== "development") {
      setOpen(true);
      return;
    }

    setSyncing(true);
    void runSync().finally(() => setSyncing(false));
  }, [runSync]);

  return (
    <>
      <Button icon={<SyncOutlined />} loading={syncing} onClick={handleClick} text="同步" />
      <IntegrationSyncModal
        defaultHome={cache.last?.home}
        onConfirm={runSync}
        onOpenChange={setOpen}
        open={open}
        targets={targets}
      />
    </>
  );
}
