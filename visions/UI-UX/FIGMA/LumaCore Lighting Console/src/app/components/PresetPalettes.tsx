import React from "react";
import { useState } from "react";
import { Move, Palette, Circle, Sparkles, Box, Layers } from "lucide-react";

interface Preset {
  id: string;
  number: number;
  name: string;
  type: 'position' | 'color' | 'beam' | 'gobo' | 'focus' | 'shaper';
}

interface PresetPalettesProps {
  onPresetSelect?: (preset: Preset) => void;
}

export function PresetPalettes({ onPresetSelect }: PresetPalettesProps) {
  const [activeType, setActiveType] = useState<Preset['type']>('position');

  const presets: Record<string, Preset[]> = {
    position: [
      { id: 'pos-1', number: 1, name: 'Center', type: 'position' },
      { id: 'pos-2', number: 2, name: 'DS Full', type: 'position' },
      { id: 'pos-3', number: 3, name: 'US Full', type: 'position' },
      { id: 'pos-4', number: 4, name: 'Stage L', type: 'position' },
      { id: 'pos-5', number: 5, name: 'Stage R', type: 'position' },
      { id: 'pos-6', number: 6, name: 'Audience', type: 'position' },
      { id: 'pos-7', number: 7, name: 'Sweep L-R', type: 'position' },
      { id: 'pos-8', number: 8, name: 'Circle', type: 'position' },
    ],
    color: [
      { id: 'col-1', number: 1, name: 'White', type: 'color' },
      { id: 'col-2', number: 2, name: 'Warm White', type: 'color' },
      { id: 'col-3', number: 3, name: 'Red', type: 'color' },
      { id: 'col-4', number: 4, name: 'Blue', type: 'color' },
      { id: 'col-5', number: 5, name: 'Green', type: 'color' },
      { id: 'col-6', number: 6, name: 'Magenta', type: 'color' },
      { id: 'col-7', number: 7, name: 'Amber', type: 'color' },
      { id: 'col-8', number: 8, name: 'Cyan', type: 'color' },
    ],
    beam: [
      { id: 'beam-1', number: 1, name: 'Wide', type: 'beam' },
      { id: 'beam-2', number: 2, name: 'Medium', type: 'beam' },
      { id: 'beam-3', number: 3, name: 'Narrow', type: 'beam' },
      { id: 'beam-4', number: 4, name: 'Spot', type: 'beam' },
      { id: 'beam-5', number: 5, name: 'Prism In', type: 'beam' },
      { id: 'beam-6', number: 6, name: 'Frost 30%', type: 'beam' },
      { id: 'beam-7', number: 7, name: 'Frost 60%', type: 'beam' },
      { id: 'beam-8', number: 8, name: 'Frost Full', type: 'beam' },
    ],
    gobo: [
      { id: 'gobo-1', number: 1, name: 'Open', type: 'gobo' },
      { id: 'gobo-2', number: 2, name: 'Breakup 1', type: 'gobo' },
      { id: 'gobo-3', number: 3, name: 'Breakup 2', type: 'gobo' },
      { id: 'gobo-4', number: 4, name: 'Stars', type: 'gobo' },
      { id: 'gobo-5', number: 5, name: 'Dots', type: 'gobo' },
      { id: 'gobo-6', number: 6, name: 'Radial', type: 'gobo' },
      { id: 'gobo-7', number: 7, name: 'Rotate Slow', type: 'gobo' },
      { id: 'gobo-8', number: 8, name: 'Rotate Fast', type: 'gobo' },
    ],
    focus: [
      { id: 'focus-1', number: 1, name: 'Sharp', type: 'focus' },
      { id: 'focus-2', number: 2, name: 'Soft Edge', type: 'focus' },
      { id: 'focus-3', number: 3, name: 'Diffuse', type: 'focus' },
      { id: 'focus-4', number: 4, name: 'Auto Track', type: 'focus' },
    ],
    shaper: [
      { id: 'shape-1', number: 1, name: 'Circle', type: 'shaper' },
      { id: 'shape-2', number: 2, name: 'Square', type: 'shaper' },
      { id: 'shape-3', number: 3, name: 'Figure 8', type: 'shaper' },
      { id: 'shape-4', number: 4, name: 'Wave', type: 'shaper' },
    ],
  };

  const typeIcons = {
    position: Move,
    color: Palette,
    beam: Circle,
    gobo: Sparkles,
    focus: Box,
    shaper: Layers,
  };

  return (
    <div className="border border-border bg-card">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-primary" />
          <span className="text-xs uppercase tracking-wider text-foreground">Preset Palettes</span>
        </div>
      </div>

      {/* Type selector tabs */}
      <div className="flex border-b border-border bg-secondary/10">
        {(['position', 'color', 'beam', 'gobo', 'focus', 'shaper'] as const).map((type) => {
          const Icon = typeIcons[type];
          const isActive = activeType === type;
          
          return (
            <button
              key={type}
              onClick={() => setActiveType(type)}
              className={`
                flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-xs uppercase tracking-wide
                border-r border-border last:border-r-0 transition-colors
                ${isActive
                  ? 'bg-primary/20 text-primary'
                  : 'text-muted-foreground hover:bg-secondary/30 hover:text-foreground'
                }
              `}
            >
              <Icon className="w-3 h-3" />
              <span className="hidden sm:inline">{type}</span>
            </button>
          );
        })}
      </div>

      {/* Preset grid */}
      <div className="p-3 bg-secondary/5">
        <div className="grid grid-cols-4 gap-2">
          {presets[activeType]?.map((preset) => (
            <button
              key={preset.id}
              onClick={() => onPresetSelect?.(preset)}
              className="
                group relative aspect-square border border-border bg-card
                hover:bg-primary/10 hover:border-primary/50 transition-all
                flex flex-col items-center justify-center p-2
              "
            >
              <div className="absolute top-1 left-1 text-xs font-mono text-muted-foreground group-hover:text-primary">
                {preset.number}
              </div>
              <div className="text-xs text-center text-foreground group-hover:text-primary">
                {preset.name}
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
