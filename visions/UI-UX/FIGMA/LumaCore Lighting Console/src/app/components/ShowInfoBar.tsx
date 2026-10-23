import React from "react";
import { useState, useEffect } from "react";
import { Cpu, User, Globe, Zap, Clock } from "lucide-react";

export function ShowInfoBar() {
  const [time, setTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="border-b border-border bg-secondary/20">
      <div className="flex items-center justify-between px-4 py-1.5 text-xs">
        {/* Left section - System info */}
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1.5">
            <Cpu className="w-3 h-3 text-muted-foreground" />
            <span className="text-muted-foreground">FW:</span>
            <span className="font-mono text-foreground">v3.2.1</span>
          </div>
          <div className="h-3 w-px bg-border" />
          <div className="flex items-center gap-1.5">
            <User className="w-3 h-3 text-muted-foreground" />
            <span className="font-mono text-primary">OPERATOR</span>
          </div>
          <div className="h-3 w-px bg-border" />
          <div className="flex items-center gap-1.5">
            <Globe className="w-3 h-3 text-muted-foreground" />
            <span className="text-muted-foreground">Show:</span>
            <span className="font-mono text-foreground">PRODUCTION_2026</span>
          </div>
        </div>

        {/* Center section - Universe status */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <Zap className="w-3 h-3 text-[#4a9e5c]" />
            <span className="text-muted-foreground">DMX:</span>
            <span className="font-mono text-[#4a9e5c]">ACTIVE</span>
          </div>
          <div className="px-2 py-0.5 border border-border bg-card font-mono">
            UNI 1-4
          </div>
          <div className="flex items-center gap-1">
            <div className="w-1.5 h-1.5 rounded-full bg-[#4a9e5c] animate-pulse" />
            <span className="text-[#4a9e5c]">512/512</span>
          </div>
        </div>

        {/* Right section - Time */}
        <div className="flex items-center gap-1.5">
          <Clock className="w-3 h-3 text-muted-foreground" />
          <span className="font-mono text-foreground tabular-nums">
            {time.toLocaleTimeString('en-US', { 
              hour: '2-digit', 
              minute: '2-digit', 
              second: '2-digit',
              hour12: false 
            })}
          </span>
          <span className="text-muted-foreground">|</span>
          <span className="font-mono text-muted-foreground">
            {time.toLocaleDateString('en-US', { 
              month: 'short', 
              day: '2-digit',
              year: 'numeric'
            })}
          </span>
        </div>
      </div>
    </div>
  );
}
