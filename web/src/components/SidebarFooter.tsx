import { Typography } from "@/components/NouiTypography";
import { useSidebarStatus } from "@/hooks/useSidebarStatus";
import { cn } from "@/lib/utils";
import { useI18n } from "@/i18n";
import { Globe } from "lucide-react";
import { useState } from "react";
import { Button } from "@nous-research/ui/ui/components/button";
import { ConnectionDialog } from "./ConnectionDialog";

export function SidebarFooter() {
  const status = useSidebarStatus();
  const { t } = useI18n();
  const [showConnection, setShowConnection] = useState(false);

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
        mondwest
        className="font-mono-ui text-[0.7rem] tabular-nums tracking-[0.1em] text-muted-foreground/70 lowercase"
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
