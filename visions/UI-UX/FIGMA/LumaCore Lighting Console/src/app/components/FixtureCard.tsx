import React from "react";
import { Lightbulb, Radio } from "lucide-react";
import { Fixture } from "../types/lighting";
import { motion } from "motion/react";

interface FixtureCardProps {
  fixture: Fixture;
  onSelect?: () => void;
  isSelected?: boolean;
}

export function FixtureCard({ fixture, onSelect, isSelected = false }: FixtureCardProps) {
  const typeColors = {
    "Moving Head": "text-primary",
    "PAR": "text-[#6e8eb8]",
    "Wash": "text-[#9a6ec4]",
    "Strobe": "text-[#c93838]",
    "Laser": "text-[#4a9e5c]",
    "LED Bar": "text-primary",
  };

  return (
    <motion.button
      onClick={onSelect}
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      className={`w-full text-left border bg-card overflow-hidden transition-colors ${
        isSelected ? 'border-primary' : 'border-border hover:border-primary/50'
      }`}
    >
      {/* Header */}
      <div className={`border-b bg-secondary/30 px-3 py-2 ${
        isSelected ? 'border-primary' : 'border-border'
      }`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={`w-1.5 h-1.5 rounded-full ${
              fixture.intensity > 0 ? 'bg-primary' : 'bg-border'
            }`} />
            <span className="text-xs uppercase tracking-wider text-foreground font-mono">
              {fixture.name}
            </span>
          </div>
          {fixture.intensity > 0 && (
            <div className="flex items-center gap-1 text-xs">
              <div className="w-1.5 h-1.5 rounded-full bg-[#4a9e5c] animate-pulse" />
              <span className="text-[#4a9e5c]">LIVE</span>
            </div>
          )}
        </div>
      </div>

      {/* Image Preview */}
      {fixture.imageUrl && (
        <div className="relative h-24 overflow-hidden border-b border-border">
          <img
            src={fixture.imageUrl}
            alt={fixture.name}
            className="w-full h-full object-cover opacity-40"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-card via-card/50 to-transparent" />
        </div>
      )}

      {/* Info Section */}
      <div className="p-3 border-b border-border bg-secondary/10 space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-muted-foreground">TYPE</span>
          <span className={`${typeColors[fixture.type]} uppercase`}>{fixture.type}</span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-muted-foreground">MANUFACTURER</span>
          <span className="text-foreground">{fixture.manufacturer}</span>
        </div>
      </div>

      {/* DMX Info */}
      <div className="p-3 bg-secondary/5">
        <div className="grid grid-cols-3 gap-2 text-xs">
          <div>
            <div className="text-muted-foreground mb-0.5">DMX</div>
            <div className="font-mono text-primary">{fixture.dmxAddress}</div>
          </div>
          <div>
            <div className="text-muted-foreground mb-0.5">UNI</div>
            <div className="font-mono text-primary">{fixture.universe}</div>
          </div>
          <div>
            <div className="text-muted-foreground mb-0.5">CH</div>
            <div className="font-mono text-primary">{fixture.channels}</div>
          </div>
        </div>

        {/* Intensity Meter */}
        <div className="mt-3">
          <div className="flex items-center justify-between text-xs mb-1">
            <span className="text-muted-foreground">INTENSITY</span>
            <span className="font-mono text-primary">{Math.round((fixture.intensity / 255) * 100)}%</span>
          </div>
          <div className="h-1 bg-secondary border border-border overflow-hidden">
            <div 
              className="h-full bg-primary transition-all duration-300"
              style={{ width: `${(fixture.intensity / 255) * 100}%` }}
            />
          </div>
        </div>
      </div>
    </motion.button>
  );
}