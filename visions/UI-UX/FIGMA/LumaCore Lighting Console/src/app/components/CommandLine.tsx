import React from "react";
import { useState } from "react";
import { Terminal, ChevronRight } from "lucide-react";

export function CommandLine() {
  const [command, setCommand] = useState("");
  const [history, setHistory] = useState<string[]>([
    "SYSTEM READY - LUMACORE v3.2.1",
  ]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (command.trim()) {
      setHistory([...history, `> ${command}`]);
      setCommand("");
    }
  };

  return (
    <div className="border border-border bg-card">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <Terminal className="w-3.5 h-3.5 text-primary" />
          <span className="text-xs uppercase tracking-wider text-foreground">Command Line</span>
        </div>
      </div>

      <div className="bg-black/40 p-3">
        {/* Command history */}
        <div className="space-y-1 mb-2 max-h-20 overflow-y-auto">
          {history.slice(-3).map((line, i) => (
            <div key={i} className="text-xs font-mono text-muted-foreground">
              {line}
            </div>
          ))}
        </div>

        {/* Input */}
        <form onSubmit={handleSubmit} className="flex items-center gap-2">
          <ChevronRight className="w-3 h-3 text-primary" />
          <input
            type="text"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            placeholder="Enter command..."
            className="flex-1 bg-transparent border-none outline-none text-xs font-mono text-foreground placeholder:text-muted-foreground/50"
          />
        </form>
      </div>
    </div>
  );
}
