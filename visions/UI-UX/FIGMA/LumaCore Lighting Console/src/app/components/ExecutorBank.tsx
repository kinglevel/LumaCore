import React from "react";
import { Play, Pause, Square } from "lucide-react";

interface Executor {
  id: string;
  number: number;
  label: string;
  cueNumber?: number;
  isActive: boolean;
  intensity: number;
}

interface ExecutorBankProps {
  executors?: Executor[];
  onExecutorTrigger?: (id: string) => void;
  onExecutorFader?: (id: string, value: number) => void;
}

export function ExecutorBank({ 
  executors = [],
  onExecutorTrigger,
  onExecutorFader 
}: ExecutorBankProps) {
  // Default executors if none provided
  const defaultExecutors: Executor[] = Array.from({ length: 10 }, (_, i) => ({
    id: `exec-${i + 1}`,
    number: i + 1,
    label: i < 3 ? `Scene ${i + 1}` : '',
    cueNumber: i < 3 ? i + 1 : undefined,
    isActive: false,
    intensity: 100,
  }));

  const displayExecutors = executors.length > 0 ? executors : defaultExecutors;

  return (
    <div className="border border-border bg-card">
      <div className="border-b border-border bg-secondary/30 px-4 py-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-primary" />
            <span className="text-xs uppercase tracking-wider text-foreground">Executor Bank</span>
          </div>
          <div className="text-xs text-muted-foreground">Page 1 / 10</div>
        </div>
      </div>

      <div className="p-4 bg-secondary/5">
        <div className="grid grid-cols-5 lg:grid-cols-10 gap-3">
          {displayExecutors.map((executor) => (
            <div key={executor.id} className="flex flex-col gap-2">
              {/* Fader */}
              <div className="relative h-24 bg-secondary border border-border">
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={executor.intensity}
                  onChange={(e) => onExecutorFader?.(executor.id, Number(e.target.value))}
                  className="absolute top-0 left-1/2 -translate-x-1/2 h-24 w-1.5 appearance-none bg-transparent cursor-pointer"
                  style={{
                    writingMode: 'bt-lr',
                    WebkitAppearance: 'slider-vertical',
                  }}
                />
                <style>{`
                  input[type="range"]::-webkit-slider-runnable-track {
                    background: transparent;
                  }
                  input[type="range"]::-webkit-slider-thumb {
                    appearance: none;
                    width: 16px;
                    height: 6px;
                    background: #d4a944;
                    cursor: pointer;
                  }
                  input[type="range"]::-moz-range-thumb {
                    width: 16px;
                    height: 6px;
                    background: #d4a944;
                    cursor: pointer;
                    border: none;
                  }
                `}</style>
              </div>

              {/* Button */}
              <button
                onClick={() => onExecutorTrigger?.(executor.id)}
                className={`
                  relative border px-2 py-3 text-xs transition-all
                  ${executor.isActive
                    ? 'border-[#4a9e5c] bg-[#4a9e5c]/20 text-[#4a9e5c]'
                    : 'border-border bg-card text-foreground hover:bg-secondary'
                  }
                `}
              >
                {executor.isActive && (
                  <div className="absolute top-1 right-1 w-1.5 h-1.5 rounded-full bg-[#4a9e5c] animate-pulse" />
                )}
                <div className="font-mono mb-1">{executor.number}</div>
                {executor.cueNumber !== undefined && (
                  <div className="text-primary font-mono text-[10px] mb-1">
                    Q{executor.cueNumber.toString().padStart(2, '0')}
                  </div>
                )}
                <div className="text-[10px] text-muted-foreground truncate">
                  {executor.label || '---'}
                </div>
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
