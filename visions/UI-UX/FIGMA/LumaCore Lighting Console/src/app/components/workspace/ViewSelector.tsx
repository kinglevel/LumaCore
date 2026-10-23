import React from "react";
import { ViewType } from "../../types/workspace";
import { 
  Layers, 
  Sliders, 
  Play, 
  Users, 
  Zap, 
  Palette, 
  Monitor, 
  Grid3x3,
  Terminal 
} from "lucide-react";

interface ViewSelectorProps {
  onSelect: (type: ViewType) => void;
  onCancel: () => void;
}

const viewOptions: { type: ViewType; label: string; icon: React.ComponentType<any>; description: string }[] = [
  { type: 'patch', label: 'Patch', icon: Layers, description: 'Fixture management & DMX addressing' },
  { type: 'cues', label: 'Cues', icon: Play, description: 'Scene management & playback' },
  { type: 'groups', label: 'Groups', icon: Users, description: 'Fixture grouping' },
  { type: 'effects', label: 'Effects', icon: Zap, description: 'Dynamic effects engine' },
  { type: 'presets', label: 'Presets', icon: Palette, description: 'Preset palettes' },
  { type: 'output-monitor', label: 'Output Monitor', icon: Monitor, description: 'DMX output monitoring' },
  { type: 'executors', label: 'Executors', icon: Grid3x3, description: 'Playback faders' },
  { type: 'command', label: 'Command', icon: Terminal, description: 'Command line interface' },
];

export function ViewSelector({ onSelect, onCancel }: ViewSelectorProps) {
  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
      <div className="bg-card border border-border max-w-3xl w-full mx-4">
        {/* Header */}
        <div className="border-b border-border bg-secondary/30 px-4 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-primary" />
              <h2 className="text-sm uppercase tracking-wider text-foreground">Select View Type</h2>
            </div>
            <button 
              onClick={onCancel}
              className="text-muted-foreground hover:text-foreground text-xs uppercase tracking-wide"
            >
              Cancel
            </button>
          </div>
        </div>

        {/* Grid of options */}
        <div className="p-6 bg-secondary/5">
          <div className="grid grid-cols-3 gap-3">
            {viewOptions.map((option) => {
              const Icon = option.icon;
              return (
                <button
                  key={option.type}
                  onClick={() => onSelect(option.type)}
                  className="border border-border bg-card hover:bg-primary/10 hover:border-primary/50 transition-all p-4 text-left group"
                >
                  <div className="flex items-start gap-3">
                    <Icon className="w-5 h-5 text-primary group-hover:text-primary flex-shrink-0 mt-0.5" />
                    <div>
                      <div className="text-sm text-foreground group-hover:text-primary mb-1 uppercase tracking-wide">
                        {option.label}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {option.description}
                      </div>
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}