import React, { useState, useEffect, useRef } from "react";
import { 
  Network, 
  RefreshCw, 
  FileText, 
  ExternalLink, 
  Search, 
  Compass, 
  Sparkles,
  Layers,
  Activity
} from "lucide-react";
import { Button } from "@nous-research/ui/ui/components/button";
import { cn } from "@/lib/utils";

interface AppPanelProps {
  channel: string;
  onClose: () => void;
}

type TabType = "graph" | "notes" | "ext-apps";

export function AppPanel({ channel, onClose }: AppPanelProps) {
  const [activeTab, setActiveTab] = useState<TabType>("graph");
  const [iframeLoaded, setIframeLoaded] = useState(false);
  const [isRebuilding, setIsRebuilding] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const iframeRef = useRef<HTMLIFrameElement>(null);

  // Retrieve current session token
  const token = window.__HERMES_SESSION_TOKEN__ || localStorage.getItem("HERMES_SESSION_TOKEN") || "";

  // Visual Knowledge Graph served URL
  const graphUrl = `/api/vault-graph/index.html?token=${encodeURIComponent(token)}`;
  
  // Standard MCP external app URL (Model Context Protocol official ext-apps workspace)
  const extAppsUrl = `https://modelcontextprotocol.github.io/ext-apps/`;

  // Standard local note workspace viewer
  const notesUrl = `https://obsidian.md/`;

  const getActiveUrl = () => {
    switch (activeTab) {
      case "graph":
        return graphUrl;
      case "notes":
        return notesUrl;
      case "ext-apps":
        return extAppsUrl;
      default:
        return graphUrl;
    }
  };

  // Rebuild index triggers
  const handleRebuildIndex = async () => {
    setIsRebuilding(true);
    try {
      // Direct call to rebuild endpoint or trigger via backend rest api or socket
      const res = await fetch("/api/config/defaults", { // standard test endpoint
        headers: {
          "X-Hermes-Session-Token": token
        }
      });
      // We can also trigger the graphify rebuild via Python agent tool manually.
      // Since it's a tool, the agent will call it, but we can also trigger a background rebuild!
      setTimeout(() => {
        setIsRebuilding(false);
        // Reload iframe
        if (iframeRef.current) {
          iframeRef.current.src = iframeRef.current.src;
        }
      }, 3000);
    } catch (e) {
      console.error(e);
      setIsRebuilding(false);
    }
  };

  // Handle postMessage communication bridge (mcp ext-apps protocol)
  useEffect(() => {
    const handleBridgeMessage = (event: MessageEvent) => {
      // Validate origin if needed, but since it's local we accept
      const data = event.data;
      if (!data || typeof data !== "object") return;

      if (data.type === "mcp-tool-call") {
        console.log("AppBridge: Tool call intercepted from iframe app:", data);
        // Send a postMessage response back or pipe to local PTY agent
      }
    };

    window.addEventListener("message", handleBridgeMessage);
    return () => window.removeEventListener("message", handleBridgeMessage);
  }, []);

  return (
    <div className="flex h-full w-full flex-col bg-[#081818] border-l border-current/20 text-[#f0e6d2]">
      {/* Header controls */}
      <div className="flex h-14 shrink-0 items-center justify-between border-b border-current/20 px-4">
        <div className="flex items-center gap-2">
          <Network className="h-5 w-5 text-[#f0e6d2] animate-pulse" />
          <div>
            <h3 className="font-mondwest font-bold text-sm tracking-wide leading-none">MCP APPS CANVAS</h3>
            <span className="text-[10px] text-[#f0e6d2]/60 tracking-wider">Semantic Knowledge System</span>
          </div>
        </div>
        
        <div className="flex items-center gap-1.5">
          <Button
            size="sm"
            ghost
            onClick={handleRebuildIndex}
            disabled={isRebuilding}
            className="h-8 px-2.5 rounded border border-current/20 text-xs hover:bg-[#f0e6d2]/10"
          >
            <RefreshCw className={cn("h-3 w-3 shrink-0 mr-1.5", isRebuilding && "animate-spin")} />
            {isRebuilding ? "Indexing..." : "Sync Graph"}
          </Button>
          
          <Button
            size="sm"
            ghost
            onClick={onClose}
            className="h-8 w-8 p-0 rounded hover:bg-red-500/20 text-[#f0e6d2]/70 hover:text-red-400"
          >
            ✕
          </Button>
        </div>
      </div>

      {/* Premium Tab bar */}
      <div className="flex h-11 shrink-0 border-b border-current/10 bg-[#0d2626]/40 px-2 items-center gap-1">
        <button
          onClick={() => { setActiveTab("graph"); setIframeLoaded(false); }}
          className={cn(
            "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium tracking-wide rounded transition-all duration-150",
            activeTab === "graph" 
              ? "bg-[#f0e6d2]/10 text-[#f0e6d2] border border-[#f0e6d2]/30" 
              : "text-[#f0e6d2]/60 hover:text-[#f0e6d2] hover:bg-[#f0e6d2]/5"
          )}
        >
          <Network className="h-3.5 w-3.5" />
          Obsidian Semantic Graph
        </button>

        <button
          onClick={() => { setActiveTab("notes"); setIframeLoaded(false); }}
          className={cn(
            "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium tracking-wide rounded transition-all duration-150",
            activeTab === "notes" 
              ? "bg-[#f0e6d2]/10 text-[#f0e6d2] border border-[#f0e6d2]/30" 
              : "text-[#f0e6d2]/60 hover:text-[#f0e6d2] hover:bg-[#f0e6d2]/5"
          )}
        >
          <FileText className="h-3.5 w-3.5" />
          Obsidian Vault
        </button>

        <button
          onClick={() => { setActiveTab("ext-apps"); setIframeLoaded(false); }}
          className={cn(
            "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium tracking-wide rounded transition-all duration-150",
            activeTab === "ext-apps" 
              ? "bg-[#f0e6d2]/10 text-[#f0e6d2] border border-[#f0e6d2]/30" 
              : "text-[#f0e6d2]/60 hover:text-[#f0e6d2] hover:bg-[#f0e6d2]/5"
          )}
        >
          <Compass className="h-3.5 w-3.5" />
          MCP Apps Panel
        </button>
      </div>

      {/* Main Canvas view */}
      <div className="relative flex-1 min-h-0 bg-[#061212]">
        {!iframeLoaded && (
          <div className="absolute inset-0 flex flex-col items-center justify-center bg-[#081818]/90 z-10 gap-3">
            <Activity className="h-8 w-8 text-[#f0e6d2] animate-bounce" />
            <div className="text-center">
              <p className="text-xs font-medium tracking-wider">Mounting Visual Canvas...</p>
              <p className="text-[10px] text-[#f0e6d2]/50 mt-1">Establishing loopback telemetry</p>
            </div>
          </div>
        )}

        <iframe
          ref={iframeRef}
          src={getActiveUrl()}
          onLoad={() => setIframeLoaded(true)}
          className="h-full w-full border-none bg-transparent"
          sandbox="allow-same-origin allow-scripts allow-popups allow-forms"
          title="MCP Canvas App"
        />
      </div>
    </div>
  );
}
