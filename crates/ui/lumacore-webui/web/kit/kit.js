// LumaCore UI kit entry (decisions §30): plain custom elements, ES modules,
// no build step, nothing from the internet. Every window, menu, dialog and
// bar is built from these elements; a missing element is designed as a mockup
// first (docs/dev/ui-kit/mockups/). The page's classic script waits for
// `window.LumaKit` (event `lumakit`) before it boots.

import * as theme from './theme.js';
import * as pointer from './pointer.js';
import { icon, loadSprite } from './icon.js';
import './button.js';
import * as menu from './menu.js';
import * as dialog from './dialog.js';
import './avatar.js';
import * as keyboard from './keyboard.js';
import * as keypad from './keypad.js';
import * as popup from './popup.js';
import * as suggest from './suggest.js';
import * as settings from './settings.js';
import './people.js';
import './surface.js';
import './overview.js';
import './vlist.js';
import './graph.js';
import * as windowBar from './windowbar.js';
import './tiles.js';
import * as grid from './grid.js';
import * as slab from './slab.js';
import * as setup from './setup.js';
import * as editCard from './editcard.js';
import './maxbar.js';
import { toast } from './toast.js';
import * as clipboard from './clipboard.js';
import * as mini from './minipane.js';
import * as split from './split.js';
import './pool.js';
import './tree.js';
import './tagspool.js';
import './pathbar.js';
import './treewindow.js';
import * as marks from './marks.js';
import './devicegrid.js';
import * as statusStrip from './statusstrip.js';
import * as themeEditor from './themeeditor.js';
import * as context from './context.js';
import * as value from './value.js';
import * as insert from './insert.js';
import * as mvrimport from './mvrimport.js';
import './dmxmap.js';
import './files.js';
import * as editPatch from './editpatch.js';
import * as tools from './tools.js';
import * as groupEditor from './groupeditor.js';
import * as attrBoard from './attrboard.js';
import './encoderbars.js';
import './fader.js';
import './encoderbar.js';
import * as oops from './oops.js';
import * as presetEditor from './preseteditor.js';
import * as cueSheet from './cuesheet.js';
import { FeedClient, SampleRing, drawLoop, applyDiff } from './feeds.js';
import { buildShowroom } from './showroom.js';

const ready = Promise.all([
  loadSprite(),
  keyboard.loadLayouts(),
  document.fonts?.ready?.catch(() => {}) ?? Promise.resolve(),
]);

window.LumaKit = { theme, pointer, icon, marks, statusStrip, themeEditor, menu, dialog, keyboard, keypad, popup, settings, windowBar, suggest, toast, clipboard, mini, split, context, value, insert, mvrimport, editPatch, tools, groupEditor, presetEditor, attrBoard, oops, cueSheet, grid, slab, setup, editCard, feeds: { FeedClient, SampleRing, drawLoop, applyDiff }, showroom: buildShowroom, ready };
ready.then(() => window.dispatchEvent(new Event('lumakit')));
