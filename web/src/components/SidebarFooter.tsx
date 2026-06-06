import { Typography } from "@nous-research/ui/ui/components/typography/index";
import type { StatusResponse } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useI18n } from "@/i18n";
import { Globe } from "lucide-react";
import { useState } from "react";
import { Button } from "@nous-research/ui/ui/components/button";
import { ConnectionDialog } from "./ConnectionDialog";

export function SidebarFooter({ status }: SidebarFooterProps) {
  const { t } = useI18n();
  const [showConnection, setShowConnection] = useState(
    typeof window !== "undefined" &&
      (!localStorage.getItem("HERMES_SESSION_TOKEN") ||
        !localStorage.getItem("HERMES_BACKEND_URL"))
  );

  return (
    <>
      {showConnection && <ConnectionDialog onClose={() => setShowConnection(false)} />}
    <div
      className={cn(
        "flex shrink-0 items-center justify-between gap-2",
        "px-5 py-2.5",
        "border-t border-current/10",
      )}
    >
      <Typography
        className="font-mono-ui text-xs tabular-nums tracking-[0.08em] text-text-tertiary lowercase"
      >
        {status?.version != null ? `v${status.version}` : "—"}
      </Typography>

      <div className="flex items-center gap-2">
        <Button
          ghost
          size="icon"
          className="h-6 w-6 text-muted-foreground/60 hover:text-midground"
          onClick={() => setShowConnection(true)}
          title="Remote Connection"
        >
          <Globe className="h-3.5 w-3.5" />
        </Button>

        <a
          href="https://nousresearch.com"
          target="_blank"
          rel="noopener noreferrer"
          className={cn(
            "font-mondwest text-[0.65rem] tracking-[0.15em] text-midground",
            "transition-opacity hover:opacity-90",
            "focus-visible:rounded-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-midground/40",
          )}
          style={{ mixBlendMode: "plus-lighter" }}
        >
          {t.app.footer.org}
        </a>
      </div>
    </div>
    </>
  );
}

interface SidebarFooterProps {
  status: StatusResponse | null;
}
