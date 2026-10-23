import React from "react";
import { ViewType } from "../../types/workspace";
import Home from "../../pages/Home";
import Cues from "../../pages/Cues";
import Groups from "../../pages/Groups";
import { GroupSelection } from "../GroupSelection";
import { EffectsEngine } from "../EffectsEngine";
import { PresetPalettes } from "../PresetPalettes";
import { ExecutorBank } from "../ExecutorBank";
import { CommandLine } from "../CommandLine";
import { Circle } from "lucide-react";
import { fixtures } from "../../data/fixtures";

interface ViewRendererProps {
  type: ViewType;
}

export function ViewRenderer({ type }: ViewRendererProps) {
  switch (type) {
    case 'patch':
      return <Home />;
    
    case 'cues':
      return <Cues />;
    
    case 'groups':
      return <Groups />;
    
    case 'effects':
      return (
        <div className="p-4">
          <EffectsEngine />
        </div>
      );
    
    case 'presets':
      return (
        <div className="p-4">
          <PresetPalettes />
        </div>
      );
    
    case 'output-monitor':
      return (
        <div className="p-4">
          <div className="border border-border bg-card">
            <div className="border-b border-border bg-secondary/30 px-4 py-2">
              <div className="flex items-center gap-2">
                <div className="w-1.5 h-1.5 rounded-full bg-[#4a9e5c] animate-pulse" />
                <span className="text-xs uppercase tracking-wider text-foreground">DMX Output Monitor</span>
              </div>
            </div>
            <div className="p-4 bg-secondary/10 max-h-[600px] overflow-y-auto">
              <div className="space-y-2">
                {fixtures.map((fixture) => (
                  <div
                    key={fixture.id}
                    className="border border-border bg-card p-3"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-xs font-mono text-foreground">{fixture.name}</span>
                      <div className="flex items-center gap-1">
                        <Circle 
                          className="w-2 h-2" 
                          fill={fixture.intensity > 0 ? "#4a9e5c" : "transparent"}
                          stroke={fixture.intensity > 0 ? "#4a9e5c" : "#6e6e78"}
                        />
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-xs">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">DMX:</span>
                        <span className="font-mono text-primary">{fixture.dmxAddress}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">INT:</span>
                        <span className="font-mono text-primary">{fixture.intensity}</span>
                      </div>
                      {fixture.type === "Moving Head" && (
                        <>
                          <div className="flex justify-between">
                            <span className="text-muted-foreground">PAN:</span>
                            <span className="font-mono text-primary">{fixture.pan}</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-muted-foreground">TILT:</span>
                            <span className="font-mono text-primary">{fixture.tilt}</span>
                          </div>
                        </>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      );
    
    case 'executors':
      return (
        <div className="p-4">
          <ExecutorBank />
        </div>
      );
    
    case 'command':
      return (
        <div className="p-4">
          <CommandLine />
        </div>
      );
    
    default:
      return (
        <div className="flex items-center justify-center h-full text-muted-foreground">
          Unknown view type: {type}
        </div>
      );
  }
}