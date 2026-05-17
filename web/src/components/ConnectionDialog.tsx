import { useState } from "react";
import { Globe, Save, X } from "lucide-react";
import { Button } from "@nous-research/ui/ui/components/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { setBackendUrl, setManualToken } from "@/lib/api";

export function ConnectionDialog({ onClose }: { onClose: () => void }) {
  const [url, setUrl] = useState(localStorage.getItem("HERMES_BACKEND_URL") || "");
  const [token, setToken] = useState(localStorage.getItem("HERMES_SESSION_TOKEN") || "");

  const handleSave = () => {
    setBackendUrl(url);
    setManualToken(token);
    onClose();
    window.location.reload(); // Reload to apply new connection settings
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
      <Card className="w-full max-w-md border-current/20 bg-background-base">
        <CardHeader className="flex flex-row items-center justify-between py-3 px-4 border-b border-current/10">
          <CardTitle className="text-sm flex items-center gap-2 font-mondwest tracking-wider">
            <Globe className="h-4 w-4" />
            REMOTE CONNECTION
          </CardTitle>
          <Button ghost size="xs" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </CardHeader>
        <CardContent className="grid gap-4 p-4">
          <div className="grid gap-2">
            <Label htmlFor="backend-url" className="text-[10px] uppercase tracking-[0.15em] opacity-60">
              Backend URL
            </Label>
            <Input
              id="backend-url"
              placeholder="http://192.168.1.xxx:9119"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              className="h-8 text-xs font-mono"
            />
            <p className="text-[10px] opacity-40 leading-tight">
              Your computer's local IP or a public tunnel URL (Cloudflare/Ngrok).
            </p>
          </div>
          <div className="grid gap-2">
            <Label htmlFor="session-token" className="text-[10px] uppercase tracking-[0.15em] opacity-60">
              Session Token
            </Label>
            <Input
              id="session-token"
              type="password"
              placeholder="Paste session token"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              className="h-8 text-xs font-mono"
            />
            <p className="text-[10px] opacity-40 leading-tight">
              Retrieve this from your computer's browser console: <br/>
              <code>window.__HERMES_SESSION_TOKEN__</code>
            </p>
          </div>
          <Button size="sm" onClick={handleSave} className="mt-2 w-full" prefix={<Save />}>
            SAVE & CONNECT
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
