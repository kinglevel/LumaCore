import React from "react";
import { useState } from "react";
import { Zap, Waves, RotateCw, TrendingUp } from "lucide-react";

interface Effect {
  id: string;
  name: string;
  type: 'chase' | 'wave' | 'circle' | 'random';
  speed: number;
  isActive: boolean;
}

export function EffectsEngine() {
  const [effects, setEffects] = useState<Effect[]>([
    { id: 'fx-1', name: 'Chase 1', type: 'chase', speed: 50, isActive: false },
    { id: 'fx-2', name: 'Wave', type: 'wave', speed: 30, isActive: false },
    { id: 'fx-3', name: 'Circle', type: 'circle', speed: 40, isActive: false },
    { id: 'fx-4', name: 'Random', type: 'random', speed: 60, isActive: false },
  ]);

  const toggleEffect = (id: string) => {
    setEffects(prev => prev.map(fx => 
      fx.id === id ? { ...fx, isActive: !fx.isActive } : fx
    ));
  };

  const updateSpeed = (id: string, speed: number) => {
    setEffects(prev => prev.map(fx => 
      fx.id === id ? { ...fx, speed } : fx
    ));
  };

  const getIcon = (type: Effect['type']) => {
    switch (type) {
      case 'chase': return TrendingUp;
      case 'wave': return Waves;
      case 'circle': return RotateCw;
      case 'random': return Zap;
    }
  };

  return (
    <div className="border border-border bg-card">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <Zap className="w-3.5 h-3.5 text-primary" />
          <span className="text-xs uppercase tracking-wider text-foreground">Effects Engine</span>
        </div>
      </div>

      <div className="p-3 bg-secondary/5 space-y-2">
        {effects.map((effect) => {
          const Icon = getIcon(effect.type);
          
          return (
            <div key={effect.id} className="border border-border bg-card p-3">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <Icon className="w-3.5 h-3.5 text-muted-foreground" />
                  <span className="text-xs text-foreground">{effect.name}</span>
                </div>
                <button
                  onClick={() => toggleEffect(effect.id)}
                  className={`
                    px-3 py-1 text-xs uppercase border transition-all
                    ${effect.isActive
                      ? 'border-[#4a9e5c] bg-[#4a9e5c]/20 text-[#4a9e5c]'
                      : 'border-border bg-secondary text-muted-foreground hover:text-foreground'
                    }
                  `}
                >
                  {effect.isActive ? 'ON' : 'OFF'}
                </button>
              </div>
              <div className="space-y-1">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-muted-foreground">Speed</span>
                  <span className="font-mono text-primary">{effect.speed}%</span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={effect.speed}
                  onChange={(e) => updateSpeed(effect.id, Number(e.target.value))}
                  className="w-full h-1 appearance-none bg-secondary border border-border cursor-pointer"
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
