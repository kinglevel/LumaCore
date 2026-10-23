---
title: Console Features
category: vision
---
# LumaCore Lighting Console - Feature Documentation

## Overview
A professional-grade lighting control console application built with React, featuring DMX control, fixture management, and scene programming capabilities. Designed to emulate the workflow and aesthetic of industry-standard consoles like ETC Eos.

## Architecture

### Core Technologies
- **React** with React Router for multi-page navigation
- **TypeScript** for type safety
- **Tailwind CSS v4** for styling
- **Motion (Framer Motion)** for animations
- **Local Storage** for data persistence

### Design Aesthetic
- Near-black backgrounds (#0a0a0c)
- Amber/golden yellow accents (#d4a944) as primary color
- Muted grays for secondary elements
- Dense, information-rich layouts
- Monospace fonts for technical data
- Minimal border radius for technical feel

## Main Views

### 1. Patch View (Home)
**Route:** `/`

**Purpose:** Fixture management and DMX addressing

**Components:**
- `PatchSpreadsheet` - Spreadsheet-style fixture table with:
  - Channel numbers
  - Fixture names and types
  - Manufacturer information
  - DMX Universe and Address
  - Channel count
  - Current intensity
  - Status indicators
  - Multi-select functionality (Ctrl/Cmd + Click)

- `GroupSelection` - Quick access to fixture groups:
  - Moving Heads
  - Stage Wash
  - Front PARs
  - LED Bars
  - All Fixtures

- `CommandLine` - Text-based command input for advanced operators

**Features:**
- Click to select individual fixtures
- Ctrl/Cmd + Click for multi-select
- Real-time status display
- DMX footprint calculation
- "Control Selected" button navigates to Live view with selected fixtures

### 2. Live Control View
**Route:** `/live?fixtures={ids}`

**Purpose:** Real-time fixture control and programming

**Layout:** Three-column grid
- Left sidebar: Groups & Effects Engine
- Center: Main control surface
- Right sidebar: Preset Palettes & DMX Output Monitor

**Components:**

#### Left Sidebar
- `GroupSelection` - Fixture grouping for batch control
- `EffectsEngine` - Dynamic effects with:
  - Chase patterns
  - Wave effects
  - Circle movements
  - Random sequences
  - Speed control (0-100%)
  - On/Off toggle per effect

#### Center Column
- `AttributeControls` - Quick attribute selection:
  - Intensity (I)
  - Position (P)
  - Color (C)
  - Beam (B)
  - Gobo (G)
  - FX (F)
  - Keyboard shortcuts shown

- **Live Control Surface** - Three modes:

  **Intensity Mode:**
  - Vertical faders for each fixture (0-255 DMX)
  - Percentage display
  - Master fader for global control
  - Real-time visual feedback

  **Position Mode:**
  - Pan/Tilt controls (0-255 DMX)
  - Individual fixture cards
  - Numeric value display
  - Horizontal sliders

  **Color Mode:**
  - HTML5 color pickers
  - Hex color display
  - Per-fixture color selection
  - Visual color preview

- `ExecutorBank` - Playback faders:
  - 10 executor slots per page
  - Combined fader + button
  - Cue assignment display
  - Active state indicators
  - Page navigation (1-10)

- `CommandLine` - Console command input

#### Right Sidebar
- `PresetPalettes` - Six palette types:
  - **Position:** Center, DS Full, US Full, Stage L/R, Audience, Sweep, Circle
  - **Color:** White, Warm White, Red, Blue, Green, Magenta, Amber, Cyan
  - **Beam:** Wide, Medium, Narrow, Spot, Prism, Frost levels
  - **Gobo:** Open, Breakups, Stars, Dots, Radial, Rotation
  - **Focus:** Sharp, Soft Edge, Diffuse, Auto Track
  - **Shaper:** Circle, Square, Figure 8, Wave
  - 8 presets per type
  - Quick-access grid layout

- **DMX Output Monitor:**
  - Real-time DMX value display
  - Active fixture indicator (green dot)
  - DMX Address, Intensity, Pan, Tilt
  - Scrollable list
  - Live updates

**Record Button:**
- Saves current fixture states to cue list
- Captures intensity, position, color, gobo values
- Stores in localStorage

### 3. Cues View
**Route:** `/cues`

**Purpose:** Scene management and playback control

**Components:**

- **Cue Stack Overview:**
  - Total cues count
  - Active cues
  - Average fade time
  - Total fixture states

- **Cue Sequence Table:**
  - Spreadsheet-style layout
  - Columns: Cue #, Name, Fade Time, Fixture Count, Created Date, Actions
  - Active cue highlighting
  - Sortable and scrollable
  - Go/Stop buttons per cue
  - Delete functionality

- **Playback Transport:**
  - Previous/Go/Next buttons
  - Central GO button (amber highlight)
  - Sequential cue playback

- `CommandLine` - Command input

## Global Components

### ShowInfoBar
Displays across all views at the top:
- **Firmware Version:** v3.2.1
- **User:** OPERATOR
- **Show Name:** PRODUCTION_2026
- **DMX Status:** ACTIVE with universe info (UNI 1-4)
- **Channel Usage:** 512/512
- **Real-time Clock:** 24-hour format with date
- Updates every second

### ConsoleTabs
Navigation between main views:
- Patch / Live / Cues
- Amber indicator dot on active tab
- Animated underline on active tab
- Keyboard-style aesthetic

## Data Models

### Fixture
```typescript
{
  id: string;
  name: string;
  type: "Moving Head" | "PAR" | "Wash" | "Strobe" | "Laser" | "LED Bar";
  manufacturer: string;
  dmxAddress: number;
  universe: number;
  channels: number;
  intensity: number;      // 0-255
  pan: number;           // 0-255
  tilt: number;          // 0-255
  color: string;         // Hex color
  gobo: number;          // Gobo wheel position
  imageUrl?: string;
}
```

### Scene (Cue)
```typescript
{
  id: string;
  name: string;
  cueNumber: number;
  fadeTime: number;      // seconds
  fixtureStates: {
    fixtureId: string;
    intensity: number;
    pan?: number;
    tilt?: number;
    color?: string;
    gobo?: number;
  }[];
  createdAt: string;     // ISO date
}
```

### Group
```typescript
{
  id: string;
  number: number;
  name: string;
  fixtureCount: number;
  color?: string;        // Visual identifier
}
```

### Preset
```typescript
{
  id: string;
  number: number;
  name: string;
  type: 'position' | 'color' | 'beam' | 'gobo' | 'focus' | 'shaper';
}
```

### Effect
```typescript
{
  id: string;
  name: string;
  type: 'chase' | 'wave' | 'circle' | 'random';
  speed: number;         // 0-100
  isActive: boolean;
}
```

### Executor
```typescript
{
  id: string;
  number: number;
  label: string;
  cueNumber?: number;
  isActive: boolean;
  intensity: number;     // 0-100
}
```

## Data Persistence

### localStorage Keys
- `lightingScenes` - Array of Scene objects

### Session State
- Active fixture selections
- Current control mode
- Active group
- Active attribute
- Executor states

## User Workflows

### Basic Fixture Control
1. Navigate to Patch view
2. Select fixtures (click or multi-select)
3. Click "Control Selected"
4. Choose attribute (Intensity/Position/Color)
5. Adjust values with faders/sliders/pickers

### Recording a Cue
1. Set fixture values in Live view
2. Click "Record" button in Intensity section
3. Scene saved to localStorage
4. Navigate to Cues view to see recorded cue

### Playing Back a Cue
1. Navigate to Cues view
2. Click "Go" button on desired cue
3. Cue becomes active (green indicator)
4. Click "Stop" to deactivate

### Using Presets
1. In Live view, select fixtures
2. Switch to desired preset type (Position/Color/etc)
3. Click preset button to apply values

### Using Effects
1. Select fixtures in Groups panel
2. In Effects Engine, toggle effect ON
3. Adjust speed slider
4. Effects apply to selected fixtures

### Command Line
1. Click into command line input
2. Type commands (syntax extensible)
3. Press Enter to execute
4. History shows last 3 commands

## Keyboard Shortcuts (Planned)
- `I` - Intensity attribute
- `P` - Position attribute
- `C` - Color attribute
- `B` - Beam attribute
- `G` - Gobo attribute
- `F` - FX attribute

## Styling Details

### Color Palette
```css
--background: #0a0a0c;          /* Near black */
--foreground: #c4c4c8;          /* Light gray text */
--card: #16161a;                /* Panel backgrounds */
--primary: #d4a944;             /* Amber accent */
--secondary: #1e1e24;           /* Dark gray */
--muted-foreground: #6e6e78;    /* Dimmed text */
--border: #2a2a32;              /* Panel borders */
--success: #4a9e5c;             /* Green status */
--destructive: #c93838;         /* Red alerts */
```

### Typography
- **Headers:** Uppercase, wide tracking
- **Values:** Monospace font for DMX/numeric data
- **Labels:** Small (xs), uppercase, muted
- **Technical data:** Font-mono class

### Spacing
- **Gaps:** 2-4 units (0.5rem - 1rem)
- **Padding:** 3-4 units for panels
- **Borders:** 1px solid, muted colors
- **Radius:** Minimal (0.25rem) for technical feel

## Performance Optimizations
- Motion animations with spring physics
- Staggered list animations (0.02-0.03s delays)
- Conditional rendering of control modes
- Efficient localStorage reads/writes
- Optimized re-renders with React state

## Future Enhancements
- Multi-universe support (currently shows 1-4)
- Advanced effects engine with customization
- Macro recording and playback
- Snapshot/blind mode programming
- MIDI/OSC control integration
- DMX output via Web Serial API
- Multi-user show files
- Cue list timing and automation
- Magic sheets (custom fixture layouts)
- 3D visualization preview
- Network DMX protocols (Art-Net, sACN)

## Browser Compatibility
- Modern browsers with CSS Grid support
- localStorage API required
- HTML5 color input support
- ES6+ JavaScript features

## Deployment
Built with Vite, optimized for production:
```bash
npm run build
```

Output in `/dist` directory, ready for static hosting.
