import React from "react";
import { Sliders, Move, Palette, Circle, Sparkles, Zap } from "lucide-react";

interface AttributeButton {
  id: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  shortcut?: string;
}

interface AttributeControlsProps {
  onAttributeSelect?: (attributeId: string) => void;
  activeAttribute?: string;
}

export function AttributeControls({ onAttributeSelect, activeAttribute }: AttributeControlsProps) {
  const attributes: AttributeButton[] = [
    { id: 'intensity', label: 'Intensity', icon: Sliders, shortcut: 'I' },
    { id: 'position', label: 'Position', icon: Move, shortcut: 'P' },
    { id: 'color', label: 'Color', icon: Palette, shortcut: 'C' },
    { id: 'beam', label: 'Beam', icon: Circle, shortcut: 'B' },
    { id: 'gobo', label: 'Gobo', icon: Sparkles, shortcut: 'G' },
    { id: 'fx', label: 'FX', icon: Zap, shortcut: 'F' },
  ];

  return (
    <div className="border border-border bg-card">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-primary" />
          <span className="text-xs uppercase tracking-wider text-foreground">Attribute Controls</span>
        </div>
      </div>

      <div className="p-3 bg-secondary/5">
        <div className="grid grid-cols-3 lg:grid-cols-6 gap-2">
          {attributes.map((attr) => {
            const Icon = attr.icon;
            const isActive = activeAttribute === attr.id;
            
            return (
              <button
                key={attr.id}
                onClick={() => onAttributeSelect?.(attr.id)}
                className={`
                  relative border px-4 py-3 transition-all
                  flex flex-col items-center gap-1
                  ${isActive
                    ? 'border-primary bg-primary/20 text-primary'
                    : 'border-border bg-card text-muted-foreground hover:bg-secondary hover:text-foreground'
                  }
                `}
              >
                {attr.shortcut && (
                  <div className="absolute top-1 right-1 text-[10px] font-mono opacity-50">
                    {attr.shortcut}
                  </div>
                )}
                <Icon className="w-4 h-4" />
                <span className="text-xs uppercase tracking-wide">{attr.label}</span>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
