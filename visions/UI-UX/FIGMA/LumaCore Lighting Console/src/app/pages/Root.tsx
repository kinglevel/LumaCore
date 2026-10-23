import React from "react";
import { Lightbulb } from "lucide-react";
import { ShowInfoBar } from "../components/ShowInfoBar";
import { WindowManager } from "../components/workspace/WindowManager";

export default function Root() {
  return (
    <div className="h-screen bg-background flex flex-col">
      {/* Console Header */}
      <header className="border-b border-border bg-card">
        {/* Show Info Bar */}
        <ShowInfoBar />
        
        <div className="border-b border-border bg-secondary/30 px-6 py-3">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <Lightbulb className="w-5 h-5 text-primary" />
              <h1 className="text-base uppercase tracking-wider text-foreground">LumaCore Control Surface</h1>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content - Window Manager */}
      <main className="flex-1 overflow-hidden">
        <WindowManager className="h-full" />
      </main>
    </div>
  );
}