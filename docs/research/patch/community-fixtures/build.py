"""Community fixture research for LumaCore's Map & clone window (2026-10-18).

Stage 1 (fetch.py) downloads JSON from https://community.smidn.com into raw/ (cached,
rate-limited, 429-aware). This file is stage 2: it reads only the cache (api.get returns
the cached copy) and writes sample.json + prints the numbers the report quotes.

Run:  python3 -I build.py            (from this folder; needs api.py + fetch.py + raw/)
"""
import sys, os, re, json, itertools, collections
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import api
from fetch import HEAVY

C = collections.Counter

# --------------------------------------------------------------------------
# GDTF 1.2 Annex A attribute names, as regexes ((n)/(m) wildcards -> \d+).
# An approximation written from the spec; good enough to tell standard from custom.
# --------------------------------------------------------------------------
N = r"\d+"
STD = [
 "Dimmer", "Pan", "Tilt", "PanRotate", "TiltRotate", "PositionEffect", "PositionEffectRate",
 "PositionEffectFade", "XYZ_[XYZ]", "Rot_[XYZ]", "Scale_[XYZ]", "Scale_XYZ",
 f"Gobo{N}", f"Gobo{N}(SelectSpin|SelectShake|SelectEffects|WheelIndex|WheelSpin|WheelShake|WheelRandom|WheelAudio|Pos|PosRotate|PosShake)",
 f"AnimationWheel{N}", f"AnimationWheel{N}(Audio|Macro|Random|SelectEffects|SelectShake|SelectSpin|Pos|PosRotate|PosShake)",
 f"AnimationSystem{N}", f"AnimationSystem{N}(Ramp|Shake|Audio|Random|Pos|PosRotate|PosShake|PosRandom|PosAudio|Macro)",
 f"MediaFolder{N}", f"MediaContent{N}", f"ModelFolder{N}", f"ModelContent{N}",
 "PlayMode", "PlayBegin", "PlayEnd", "PlaySpeed",
 f"ColorEffects{N}", f"Color{N}", f"Color{N}(WheelIndex|WheelSpin|WheelRandom|WheelAudio)",
 "ColorAdd_(R|G|B|C|M|Y|RY|GY|GC|BC|BM|RM|W|WW|CW|UV)", "ColorSub_(R|G|B|C|M|Y)",
 f"ColorMacro{N}", f"ColorMacro{N}Rate", "CTO", "CTC", "CTB", "Tint",
 "HSB_(Hue|Saturation|Brightness|Quality)", "CIE_(X|Y|Brightness)",
 "ColorRGB_(Red|Green|Blue|Cyan|Magenta|Yellow|Quality)",
 "VideoBoost_[RGB]", "VideoHueShift", "VideoSaturation", "VideoBrightness", "VideoContrast",
 "VideoKeyColor_[RGB]", "VideoKeyIntensity", "VideoKeyTolerance",
 "StrobeDuration", "StrobeRate", "StrobeFrequency",
 "StrobeMode(Shutter|Strobe|Pulse|PulseOpen|PulseClose|Random|RandomPulse|RandomPulseOpen|RandomPulseClose|Effect)",
 f"Shutter{N}", f"Shutter{N}(Strobe|StrobePulse|StrobePulseClose|StrobePulseOpen|StrobeRandom|StrobeRandomPulse|StrobeRandomPulseClose|StrobeRandomPulseOpen|StrobeEffect)",
 "Iris", "IrisStrobe", "IrisStrobeRandom", "IrisPulseClose", "IrisPulseOpen", "IrisRandomPulseClose", "IrisRandomPulseOpen",
 f"Frost{N}", f"Frost{N}(PulseOpen|PulseClose|Ramp)",
 f"Prism{N}", f"Prism{N}(SelectSpin|Macro|Pos|PosRotate)",
 f"Effects{N}", f"Effects{N}(Rate|Fade|Pos|PosRotate)", f"Effects{N}Adjust{N}", "EffectsSync",
 "BeamShaper", "BeamShaperMacro", "BeamShaperPos", "BeamShaperPosRotate",
 "Zoom", "ZoomModeSpot", "ZoomModeBeam", "DigitalZoom", f"Focus{N}", f"Focus{N}(Adjust|Distance)",
 f"Control{N}", "DimmerMode", "DimmerCurve", "BlackoutMode", "LEDFrequency", "LEDZoneMode", "PixelMode",
 "PanMode", "TiltMode", "PanTiltMode", "PositionModes", f"Gobo{N}WheelMode", "GoboWheelShortcutMode",
 f"AnimationWheel{N}Mode", "AnimationWheelShortcutMode", f"Color{N}Mode", "ColorWheelShortcutMode",
 "CyanMode", "MagentaMode", "YellowMode", "ColorMixMode", "ChromaticMode", "ColorCalibrationMode",
 "ColorConsistency", "ColorControl", "ColorModelMode", "ColorSettingsReset", "ColorUniformity", "CRIMode",
 "CustomColor", "UVStability", "WavelengthCorrection", "WhiteCount", "StrobeMode", "ZoomMode", "FocusMode",
 "IrisMode", f"FanMode", "FollowSpotMode", "BeamEffectIndexRotateMode", "IntensityMSpeed", "PositionMSpeed",
 "ColorMixMSpeed", "ColorWheelSelectMSpeed", f"GoboWheel{N}MSpeed", "IrisMSpeed", f"Prism{N}MSpeed",
 "FocusMSpeed", f"Frost{N}MSpeed", "ZoomMSpeed", "FrameMSpeed", "GlobalMSpeed", "ReflectorAdjust",
 "FixtureGlobalReset", "DimmerReset", "ShutterReset", "BeamReset", "ColorMixReset", "ColorWheelReset",
 "FocusReset", "FrameReset", "GoboWheelReset", "IntensityReset", "IrisReset", "PositionReset", "PanReset",
 "TiltReset", "ZoomReset", "CTBReset", "CTOReset", "CTCReset", "AnimationSystemReset",
 "FixtureCalibrationReset", "Function", "LampControl", "DisplayIntensity", "DMXInput", "NoFeature",
 "Dummy", f"Blower{N}", f"Fan{N}", f"Fog{N}", f"Haze{N}", "LampPowerMode", "Fans",
 f"Blade{N}[AB]", f"Blade{N}Rot", "ShaperRot", "ShaperMacros", "ShaperMacrosSpeed",
 f"BladeSoft{N}[AB]", f"KeyStone{N}[AB]", "Video", f"VideoEffect{N}Type", f"VideoEffect{N}Parameter{N}",
 f"VideoCamera{N}", f"VideoSoundVolume{N}", "VideoBlendMode", "InputSource", "FieldOfView",
]
STD_RE = re.compile("^(" + "|".join(STD) + ")$")
def is_std(a): return bool(a) and bool(STD_RE.match(a))

# --------------------------------------------------------------------------
# Colour system
# --------------------------------------------------------------------------
EMIT = {"ColorAdd_R": "R", "ColorAdd_G": "G", "ColorAdd_B": "B", "ColorAdd_W": "W", "ColorAdd_WW": "WW",
        "ColorAdd_CW": "CW", "ColorAdd_RY": "A", "ColorAdd_GY": "L", "ColorAdd_C": "C", "ColorAdd_UV": "UV",
        "ColorAdd_M": "M", "ColorAdd_Y": "Y", "ColorAdd_BC": "Cy", "ColorAdd_GC": "GC", "ColorAdd_BM": "BM",
        "ColorAdd_RM": "RM"}
def colour_system(attrs):
    a = set(attrs); parts = []
    if {"ColorSub_C", "ColorSub_M", "ColorSub_Y"} & a: parts.append("CMY")
    em = [EMIT[x] for x in EMIT if x in a]
    if em: parts.append("+".join(em))
    if any(x.startswith("ColorRGB_") for x in a): parts.append("ColorRGB(virtual mix)")
    if any(x.startswith("HSB_") for x in a): parts.append("HSB")
    if any(x.startswith("CIE_") for x in a): parts.append("CIE")
    wh = sorted(x for x in a if re.fullmatch(r"Color\d+", x))
    if wh: parts.append(f"wheel x{len(wh)}")
    for t in ("CTO", "CTC", "CTB", "Tint"):
        if t in a: parts.append(t)
    if any(re.fullmatch(r"ColorMacro\d+", x) for x in a): parts.append("macro")
    return " | ".join(parts) or "none"

def colour_family(attrs):
    """coarse: CMY / RGB-ish emitters / wheel-only / RGB virtual"""
    a = set(attrs); f = set()
    if {"ColorSub_C", "ColorSub_M", "ColorSub_Y"} & a: f.add("CMY")
    if any(x in a for x in EMIT): f.add("emitters")
    if any(x.startswith("ColorRGB_") for x in a): f.add("virtualRGB")
    if any(re.fullmatch(r"Color\d+", x) for x in a): f.add("wheel")
    return f

# --------------------------------------------------------------------------
# Wheels
# --------------------------------------------------------------------------
WHITE = re.compile(r"^0\.31\d*,0\.32\d*,")
def wheel_summary(wheels):
    out = []
    for name, slots in (wheels or {}).items():
        coloured = sum(1 for s in slots if s.get("color") and not WHITE.match(s["color"].replace(" ", "")))
        media = sum(1 for s in slots if s.get("media"))
        n = name.lower()
        kind = ("colour" if "color" in n or "colour" in n else "gobo" if "gobo" in n else "prism" if "prism" in n
                else "animation" if "anim" in n else "macro" if "macro" in n else "other")
        out.append({"name": name, "kind": kind, "slots": len(slots), "coloured_slots": coloured, "media_slots": media})
    return out

# --------------------------------------------------------------------------
# Geometry / instance groups
# --------------------------------------------------------------------------
def fam(g):
    return re.sub(r"\s+", " ", re.sub(r"\d+", "#", g or "")).strip()

def instance_groups(chs):
    """A repeated part = geometries with the same attribute multiset AND the same name family
    (name with digits -> #), occurring >= 2 times. Lone geometries with the same attribute
    multiset are NOT merged (they may be a real separate part, e.g. JDC1's back plate) but are
    counted as 'odd_siblings': instances a name rule would miss (Robe's 'Lens1' next to
    'P2 Zone2' ... 'P19 Zone2')."""
    by_geo = collections.defaultdict(list)
    for c in chs:
        by_geo[c["geometry"]].append(c["attribute"] or "?")
    keyed = collections.defaultdict(list)
    for g, al in by_geo.items():
        keyed[(tuple(sorted(al)), fam(g))].append(g)
    groups = []
    for (s, fm), gs in keyed.items():
        if len(gs) < 2: continue
        odd = [g for (s2, fm2), gs2 in keyed.items() if s2 == s and fm2 != fm and len(gs2) == 1 for g in gs2]
        groups.append({"count": len(gs), "attrs": sorted(set(s)), "family": fm,
                       "names": gs[:3] + (["…"] if len(gs) > 3 else []),
                       "odd_siblings": odd})
    return groups, by_geo

def mode_summary(mo, wheels):
    chs = mo["channels"]
    real = [c for c in chs if not c["virtual"]]
    virt = [c for c in chs if c["virtual"]]
    attrs = sorted(set(c["attribute"] or "?" for c in chs))
    per_attr = C(c["attribute"] or "?" for c in chs)
    per_attr_real = C(c["attribute"] or "?" for c in real)
    geos_per_attr = {a: len(set(c["geometry"] for c in chs if (c["attribute"] or "?") == a)) for a in attrs}
    groups, by_geo = instance_groups(chs)
    virt_by_attr = C(c["attribute"] for c in virt)
    return {
        "name": mo["name"], "channels": mo["channel_count"], "footprint": mo["dmx_footprint"],
        "rows": len(chs), "real_rows": len(real), "virtual_rows": len(virt),
        "virtual_attrs": dict(virt_by_attr),
        "bit16": sum(1 for c in real if (c["resolution"] or 1) >= 2),
        "bit24plus": sum(1 for c in real if (c["resolution"] or 1) >= 3),
        "breaks": sorted(set(c["dmx_break"] for c in chs if c["dmx_break"] is not None)),
        "attrs": attrs, "distinct_attrs": len(attrs),
        "categories": dict(C(c["attribute_category"] for c in chs)),
        "multi_instance_attrs": {a: n for a, n in geos_per_attr.items() if n > 1},
        "geometries": len(by_geo),
        "instance_groups": groups,
        "colour": colour_system(attrs),
        "nonstandard_attrs": [a for a in attrs if not is_std(a)],
        "null_attrs": sum(1 for c in chs if not c["attribute"]),
        "_per_attr": per_attr, "_geos": set(by_geo), "_res": {c["attribute"]: c["resolution"] for c in real},
        "_virt": set(virt_by_attr), "_cat": {c["attribute"]: c["attribute_category"] for c in chs},
        "_geo_of": {a: sorted(set(c["geometry"] for c in chs if c["attribute"] == a)) for a in attrs},
    }

def public(ms):
    return {k: v for k, v in ms.items() if not k.startswith("_")}

def load_fixture(fid):
    d = api.get(f"/api/fixtures/{fid}")
    m = api.get(f"/api/fixtures/{fid}/modes")
    if not m.get("ok"):
        return None
    sp = api.get(f"/api/fixtures/{fid}/spectral") if fid in HEAVY else {"ok": False}
    modes = [mode_summary(mo, m.get("wheels")) for mo in m["modes"]]
    emitters = [e["name"] for e in (sp.get("spectral") or {}).get("emitters", [])] if sp.get("ok") else []
    return {"id": fid, "label": HEAVY.get(fid), "manufacturer": d.get("manufacturer"), "fixture": d.get("fixture"),
            "revision": d.get("revision_name"), "wheels": wheel_summary(m.get("wheels")),
            "emitters": emitters, "modes": modes}

# --------------------------------------------------------------------------
# Mode -> mode (same fixture)
# --------------------------------------------------------------------------
def diff_modes(A, B):
    a, b = set(A["attrs"]), set(B["attrs"])
    common = a & b
    issues = C()
    lost = sorted(a - b)
    if lost: issues["attr_lost"] += 1
    collapse, expand, res, virt = [], [], [], []
    for x in common:
        na, nb = len(A["_geo_of"][x]), len(B["_geo_of"][x])
        if na > nb: collapse.append(f"{x} {na}->{nb}")
        if na < nb: expand.append(f"{x} {na}->{nb}")
        ra, rb = A["_res"].get(x), B["_res"].get(x)
        if ra and rb and ra != rb: res.append(f"{x} {ra*8}->{rb*8}bit")
        if (x in A["_virt"]) != (x in B["_virt"]): virt.append(x)
    # geometry identity: for common attrs, do the geometry NAMES carrying them survive?
    geo_rename = [x for x in common if not (set(A["_geo_of"][x]) & set(B["_geo_of"][x]))]
    if collapse: issues["instances_collapse"] += 1
    if expand: issues["instances_expand"] += 1
    if res: issues["resolution_change"] += 1
    if virt: issues["virtual_change"] += 1
    if geo_rename: issues["geometry_renamed"] += 1
    if colour_family(A["attrs"]) != colour_family(B["attrs"]): issues["colour_family_change"] += 1
    return {"from": A["name"], "to": B["name"], "lost": lost, "new": sorted(b - a), "collapse": collapse,
            "expand": expand, "resolution": res, "virtual_change": virt, "geo_renamed_attrs": len(geo_rename),
            "common_attrs": len(common), "issues": sorted(issues)}

# --------------------------------------------------------------------------
# Type -> type (two fixtures)
# --------------------------------------------------------------------------
COLOUR_ATTR = re.compile(r"^(ColorAdd_.*|ColorSub_.*|ColorRGB_.*|Color\d+|CTO|CTC|CTB|Tint|ColorMacro\d+|HSB_.*|CIE_.*)$")
NOISE_ATTR = re.compile(r"^(NoFeature\d*|Dummy\d*|.*MSpeed|.*MTime|.*Reset|Control\d+|LampControl|DisplayIntensity|"
                        r"LEDFrequency.*|IntensityIndication|DimmerMode|DimmerCurve|.*Mode|Function|FreqTweak|P3DMX)$")
def base(a): return re.sub(r"\d+", "", a or "")

def match_types(A, B):
    """Tiers, first hit wins, for each source attribute:
       exact      same GDTF attribute name
       colour     a colour attribute while the target has any colour attribute (converted as a whole)
       sibling    same name but another number (Gobo3 -> Gobo2, Prism2 -> Prism1, Effects2 -> Effects1)
       noise      a control/placeholder value (NoFeature, *MSpeed, *Reset, Control1, *Mode): never carried
       group      an unmatched target attribute exists in the same feature group (suggestion only)
       none       kept unused"""
    a, b = set(A["attrs"]) - {"?"}, set(B["attrs"]) - {"?"}
    tgt_colour = any(COLOUR_ATTR.match(x) for x in b)
    ub = b - a
    cat_a, cat_b = A["_cat"], B["_cat"]
    tiers = collections.defaultdict(list); group_cands = {}
    for x in sorted(a):
        if x in b: tiers["exact"].append(x); continue
        if COLOUR_ATTR.match(x) and tgt_colour: tiers["colour"].append(x); continue
        sib = [y for y in ub if base(y) == base(x) and y != x]
        if sib and re.search(r"\d", x): tiers["sibling"].append(f"{x}->{sib[0]}"); continue
        if NOISE_ATTR.match(x): tiers["noise"].append(x); continue
        cands = [y for y in ub if cat_b.get(y) == cat_a.get(x) and cat_a.get(x) not in (None, "Other")
                 and not NOISE_ATTR.match(y)]
        if cands: tiers["group"].append(x); group_cands[x] = cands[:4]; continue
        tiers["none"].append(x)
    inst = {x: f"{len(A['_geo_of'][x])}->{len(B['_geo_of'][x])}" for x in tiers["exact"]
            if len(A["_geo_of"][x]) != len(B["_geo_of"][x])}
    naive_group = sum(1 for x in a - b if any(cat_b.get(y) == cat_a.get(x) for y in ub) and cat_a.get(x) != "Other")
    return {"from": A["name"], "to": B["name"], "src_channels": A["channels"], "dst_channels": B["channels"],
            "src_attrs": len(a), "dst_attrs": len(b),
            "tiers": {k: len(v) for k, v in tiers.items()}, "tier_lists": dict(tiers), "group_candidates": group_cands,
            "naive_same_group_suggestions": naive_group,
            "target_unfilled": len(ub), "instance_mismatch": inst,
            "colour_src": A["colour"], "colour_dst": B["colour"],
            "colour_family_same": colour_family(A["attrs"]) == colour_family(B["attrs"])}

TYPE_PAIRS = [
 ("gdtf:146138", "gdtf:135703", "discharge profile -> discharge profile"),
 ("gdtf:116608", "gdtf:78256", "LED profile -> LED profile"),
 ("gdtf:150913", "gdtf:44146", "hybrid -> hybrid"),
 ("gdtf:159992", "gdtf:62324", "beam -> beam/hybrid"),
 ("gdtf:40784", "gdtf:91587", "aura wash -> plain wash"),
 ("gdtf:119729", "gdtf:159073", "ring+pixel wash -> ring+pixel wash"),
 ("gdtf:159072", "gdtf:60186", "aura wash -> flower wash"),
 ("gdtf:56156", "gdtf:146713", "batten 12 cells -> 16 cells"),
 ("gdtf:162563", "gdtf:119736", "tilt bar 20 px -> 18 tilting heads"),
 ("gdtf:103537", "gdtf:103011", "hybrid strobe -> hybrid strobe"),
 ("gdtf:142714", "gdtf:147974", "12 tilting heads -> 5 tilting heads"),
 ("gdtf:142714", "gdtf:119736", "tilting heads -> cells on one tilt"),
 ("gdtf:59461", "gdtf:162563", "revision change (X4 Bar 20)"),
 ("gdtf:145478", "gdtf:162855", "revision change (HalupixDUO)"),
]

def pick(modes, how):
    """max = the richest mode (most channels); std = the richest mode WITHOUT repeated parts
    (the 'one fixture' mode a programmer usually patches), else the smallest mode."""
    if how == "max":
        return max(modes, key=lambda m: (m["channels"], m["distinct_attrs"]))
    plain = [m for m in modes if not m["instance_groups"]]
    return max(plain, key=lambda m: (m["distinct_attrs"], m["channels"])) if plain else min(modes, key=lambda m: m["channels"])

# --------------------------------------------------------------------------
# Part-kind rules (proposal) + hand labels for the heavy sample
# --------------------------------------------------------------------------
RING_N = re.compile(r"ring|aura|halo|outer|background|backlight|filament|surround|corona", re.I)
ZONE_N = re.compile(r"zone|segment|seg\b|section|set\b", re.I)
HEAD_N = re.compile(r"\bhead\b|\(head|head_", re.I)
LIGHT = re.compile(r"^(Dimmer|ColorAdd_.*|ColorRGB_.*|ColorSub_.*|Color\d+.*|CTC|CTO|CTB|Shutter\d+.*|HSB_.*|CIE_.*)$")
POS = {"Pan", "Tilt", "PanRotate", "TiltRotate"}
GRID_F = re.compile(r"matrix|panel|grid|tile|cube|\d+ ?x ?\d+", re.I)
LIN_F = re.compile(r"bar\b|batten|strip|tube|line|linear|sceptron|fatron|curve|\d{3,4} ?mm|sixbar", re.I)
CIRC_F = re.compile(r"aura|ring|spiider|eye|halo|cercle|circle|round", re.I)
HEX = {7, 19, 37, 61, 91, 127}
SQUARES = {k * k for k in range(3, 40)}

NESTED = re.compile(r"\(.*#.*\).*#")   # "CW Beam(Head 3) 7": an index inside a parent's index

def classify(fixture_name, group):
    """Proposed part-kind rules -> (kind, shape). kind: Heads | Ring | Cells | Mechanical.
    Order matters; first hit wins.
      K1 own Pan/Tilt, or the name says head (and is not nested below a head) -> Heads
      K2 no light attribute at all (Zoom x2, Flower motor)                   -> Mechanical
      K3 name says ring/aura/halo/background/filament, or a zone/segment
         name with <= 6 instances on a fixture that is not a bar          -> Ring
      K4 everything else                                                   -> Cells
      S1 fixture name says matrix/panel/grid/tile/AxB                      -> Grid
      S2 fixture name says bar/batten/strip/tube/line/curve/…mm             -> Linear
      S3 Ring, or the count (+ odd-named siblings) is 7/19/37/61/91/127,
         or the fixture name says aura/ring/eye/halo/circle                -> Circle
      S4 count is a square >= 25                                           -> Grid
      S5 Heads                                                             -> Linear
      else                                                                 -> ? (ask)"""
    attrs = set(group["attrs"]); n = group["count"]; fam_ = group["family"]
    n_all = n + len(group.get("odd_siblings", []))
    nested = bool(NESTED.search(fam_))
    if (attrs & POS) or (HEAD_N.search(fam_) and not nested):
        kind = "Heads"
    elif not any(LIGHT.match(a) for a in attrs):
        kind = "Mechanical"
    elif RING_N.search(fam_) or (ZONE_N.search(fam_) and n <= 6 and not LIN_F.search(fixture_name)):
        kind = "Ring"
    else:
        kind = "Cells"
    if kind == "Mechanical":
        shape = None
    elif GRID_F.search(fixture_name):
        shape = "Grid"
    elif LIN_F.search(fixture_name):
        shape = "Linear"
    elif kind == "Ring" or n_all in HEX or CIRC_F.search(fixture_name):
        shape = "Circle"
    elif n_all in SQUARES and n_all >= 25:
        shape = "Grid"
    elif kind == "Heads":
        shape = "Linear"
    else:
        shape = "?"
    return kind, shape

# Hand labels for every repeated part in each heavy fixture's richest mode, from product
# knowledge (layout of the real unit). shape None = not sure -> only the kind is scored.
L = lambda k, s=None: (k, s)
LABELS = {
 "gdtf:159072": {"Beam Pixel #": L("Cells", "Circle"), "Aura Filament #": L("Ring", "Circle")},
 "gdtf:159073": {"Beam Pixel #": L("Cells", "Circle"), "Aura Pixel #": L("Ring", "Circle")},
 "gdtf:119729": {"P# Zone#": L("Cells", "Circle")},
 "gdtf:119736": {"P# Flower": L("Cells", "Grid"), "Zoom #": L("Mechanical"), "P#": L("Cells", "Grid")},
 "gdtf:119735": {"P#": L("Cells", "Linear")},
 "gdtf:103537": {"M# Single Plate #": L("Cells", "Linear"), "M# Single Beam #": L("Cells", "Linear")},
 "gdtf:150958": {"Beam Seg #": L("Cells", "Linear"), "Plate Seg #": L("Cells")},
 "gdtf:162563": {"SPix LED #": L("Cells", "Linear")},
 "gdtf:22993": {"SPix LED #": L("Cells", "Linear")},
 "gdtf:59461": {"SPix LED #": L("Cells", "Linear")},
 "gdtf:146713": {"Zoom_#": L("Mechanical"), "Beam_#": L("Cells", "Linear")},
 "gdtf:89307": {"Beam_#": L("Cells", "Linear")},
 "gdtf:56156": {"#/#/# Beam #": L("Cells", "Linear")},
 "gdtf:144065": {"Pixel #": L("Cells", "Linear")},
 "gdtf:149953": {"LED #": L("Cells", "Grid")},
 "gdtf:62991": {"#/# Pixel #": L("Cells", "Linear")},
 "gdtf:74459": {"pixel #": L("Cells", "Circle")},
 "gdtf:119741": {"P# Zone #": L("Cells", "Circle")},
 "gdtf:162855": {"Pix##": L("Cells", "Grid"), "B##": L("Cells", "Grid")},
 "gdtf:145478": {"Pix##": L("Cells", "Grid"), "B##": L("Cells", "Grid")},
 "gdtf:160435": {"LED #": L("Cells", "Circle"), "Reference #": L("Cells")},
 "gdtf:139321": {"Segment#of#": L("Cells", "Linear")},
 "gdtf:60186": {"Zone #": L("Ring", "Circle")},
 "gdtf:113182": {"Zone #": L("Ring", "Circle")},
 "gdtf:91587": {"Beam_#": L("Ring")},
 "gdtf:150111": {"Lens RGBL #": L("Cells", "Circle")},
 "gdtf:89324": {"Beam_#": L("Cells")},
 "gdtf:37128": {"Ring #": L("Ring", "Circle")},
 "gdtf:74383": {"Cell # Mode# RGBW": L("Cells")},
 "gdtf:142714": {"Head_#": L("Heads", "Linear")},
 "gdtf:147974": {"Head #": L("Heads", "Linear"), "Body(Head #)": L("Heads", "Linear"),
                 "Center(Head #)": L("Heads", "Linear"), "CW Beam(Head #) #": L("Cells"),
                 "WW Beam(Head #) #": L("Cells"), "CW(Head #)": L("Heads", "Linear"), "WW(Head #)": L("Heads", "Linear")},
 "gdtf:160431": {"LED #": L("Cells", "Circle"), "Reference #": L("Cells")},
 "gdtf:147669": {"Led #": L("Cells", "Circle"), "Dot #": L("Ring", "Circle")},
 "gdtf:113709": {"P# RGB": L("Cells", "Linear"), "St #": L("Cells", "Linear")},
}

# --------------------------------------------------------------------------
# Naming in the wild (library-wide facets)
# --------------------------------------------------------------------------
def naming_facets():
    d = api.get("/api/attributes", {"sources": "gdtf"})
    out = {"by_group": {}, "patterns": {}}
    tot_occ, tot_std_occ, all_names = 0, 0, []
    for c in d["facets"]["gdtf"]["categories"]:
        names = c["attributes"]
        occ = sum(a["count"] for a in names)
        std_occ = sum(a["count"] for a in names if is_std(a["name"]))
        nonstd = [a for a in names if not is_std(a["name"])]
        out["by_group"][c["name"]] = {
            "fixture_revisions_with_group": c["count"], "distinct_names": len(names),
            "distinct_nonstandard": len(nonstd), "occurrences": occ,
            "nonstandard_occ_pct": round(100 * (occ - std_occ) / occ, 1) if occ else 0,
            "top_nonstandard": [(a["name"], a["count"]) for a in sorted(nonstd, key=lambda a: -a["count"])[:12]]}
        tot_occ += occ; tot_std_occ += std_occ; all_names += names
    out["total_occ"], out["std_occ"] = tot_occ, tot_std_occ
    out["distinct_names"] = len(all_names)
    out["distinct_std"] = sum(1 for a in all_names if is_std(a["name"]))
    pat = {
        "Gobo1": r"Gobo1", "Gobo2": r"Gobo2", "Gobo3": r"Gobo3",
        "ColorAdd_*": r"ColorAdd_.*", "ColorRGB_*": r"ColorRGB_.*", "ColorSub_*": r"ColorSub_.*",
        "Effects<n>*": r"Effects\d+.*", "Control<n>": r"Control\d+", "NoFeature": r"NoFeature",
        "Shutter2+": r"Shutter[2-9]\d*", "Color2+": r"Color[2-9]", "Dimmer-like custom": r"(?!Dimmer$)(?i:.*dim.*)",
        "names with spaces": r".*\s.*",
    }
    names_cnt = {a["name"]: a["count"] for a in all_names}
    for k, rx in pat.items():
        r = re.compile("^" + rx + "$")
        hits = {n: c for n, c in names_cnt.items() if r.match(n) and (k != "Dimmer-like custom" or not is_std(n))}
        out["patterns"][k] = {"distinct": len(hits), "occurrences": sum(hits.values())}
    return out

# --------------------------------------------------------------------------
def main():
    heavy = {}
    for fid in HEAVY:
        f = load_fixture(fid)
        if f: heavy[fid] = f
    broad_meta = json.load(open(os.path.join(HERE, "broad_ids.json")))
    broad = {}
    for fid, meta in broad_meta.items():
        m = api.get(f"/api/fixtures/{fid}/modes")
        if not m.get("ok") or not m.get("modes"): continue
        broad[fid] = {"id": fid, "manufacturer": meta["manufacturer"], "fixture": meta["fixture"],
                      "via": meta["via"], "wheels": wheel_summary(m.get("wheels")),
                      "modes": [mode_summary(mo, m.get("wheels")) for mo in m["modes"]]}
    R = {"heavy": {}, "mode_diffs": {}, "type_pairs": [], "broad": {}, "naming": naming_facets(), "parts": {}}

    # 1. heavy dump
    for fid, f in heavy.items():
        R["heavy"][fid] = {**{k: v for k, v in f.items() if k != "modes"},
                           "modes": [public(m) for m in f["modes"]]}
    # 2. mode diffs (every ordered pair of modes inside a fixture, heavy sample)
    agg = C(); npairs = 0; per_fixture_issue = collections.defaultdict(C)
    for fid, f in heavy.items():
        ds = []
        for A, B in itertools.permutations(f["modes"], 2):
            d = diff_modes(A, B); ds.append(d); npairs += 1
            for i in d["issues"]: agg[i] += 1; per_fixture_issue[i][fid] += 1
        R["mode_diffs"][fid] = ds
    share = collections.defaultdict(list)
    for fid, ds in R["mode_diffs"].items():
        if not ds: continue
        for k in ("attr_lost", "instances_collapse", "instances_expand", "resolution_change",
                  "virtual_change", "geometry_renamed", "colour_family_change"):
            share[k].append(sum(k in d["issues"] for d in ds) / len(ds))
    R["mode_diff_totals"] = {"ordered_pairs": npairs, "issues": dict(agg),
                             "mean_share_of_pairs_per_fixture": {k: round(100 * sum(v) / len(v), 1) for k, v in share.items()},
                             "fixtures_with_issue": {k: len(v) for k, v in per_fixture_issue.items()},
                             "fixtures_multi_mode": sum(1 for f in heavy.values() if len(f["modes"]) > 1)}
    # 3. type pairs
    for a, b, why in TYPE_PAIRS:
        if a not in heavy or b not in heavy: continue
        for how in ("std", "max"):
            r = match_types(pick(heavy[a]["modes"], how), pick(heavy[b]["modes"], how))
            r.update({"pair": why, "src": heavy[a]["label"], "dst": heavy[b]["label"], "mode_pick": how,
                      "src_groups": [(g["count"], g["family"]) for g in pick(heavy[a]["modes"], how)["instance_groups"]],
                      "dst_groups": [(g["count"], g["family"]) for g in pick(heavy[b]["modes"], how)["instance_groups"]],
                      "src_wheels": [(w["name"], w["slots"]) for w in heavy[a]["wheels"] if w["kind"] in ("colour", "gobo")],
                      "dst_wheels": [(w["name"], w["slots"]) for w in heavy[b]["wheels"] if w["kind"] in ("colour", "gobo")]})
            R["type_pairs"].append(r)
    # 4. broad sample stats
    allf = {**broad, **{k: v for k, v in heavy.items()}}
    st = C(); modes_n = 0; ig_total = 0; ig_consistent = 0
    fam_names = C(); per_fix = []
    for fid, f in allf.items():
        ms = f["modes"]; modes_n += len(ms)
        any_nonstd = any(m["nonstandard_attrs"] for m in ms)
        any_other = any(m["categories"].get("Other") for m in ms)
        any_virtual = any(m["virtual_rows"] for m in ms)
        any_groups = any(m["instance_groups"] for m in ms)
        multi_break = any(len(m["breaks"]) > 1 for m in ms)
        st["fixtures"] += 1
        st["nonstd"] += any_nonstd; st["other"] += any_other; st["virtual"] += any_virtual
        st["multi_instance"] += any_groups; st["multi_break"] += multi_break
        st["multi_mode"] += len(ms) > 1
        st["has_wheels"] += bool([w for w in f["wheels"] if w["kind"] == "colour"])
        cw = [w for w in f["wheels"] if w["kind"] == "colour" and w["slots"] > 1]
        st["colour_wheel_without_slot_colours"] += any(w["coloured_slots"] == 0 for w in cw)
        mm = ms[:30]
        st["attr_on_other_geometry_between_modes"] += any(
            not (set(x["_geo_of"][a]) & set(y["_geo_of"][a]))
            for x, y in itertools.combinations(mm, 2) for a in set(x["attrs"]) & set(y["attrs"]))
        st["instance_count_differs_between_modes"] += any(
            len(x["_geo_of"][a]) != len(y["_geo_of"][a])
            for x, y in itertools.combinations(mm, 2) for a in set(x["attrs"]) & set(y["attrs"]))
        gobo = set(itertools.chain.from_iterable(m["attrs"] for m in ms))
        if "Gobo1" in gobo and "Gobo2" in gobo:
            r1 = bool({"Gobo1Pos", "Gobo1PosRotate"} & gobo); r2 = bool({"Gobo2Pos", "Gobo2PosRotate"} & gobo)
            st["gobo_pair"] += 1
            st["gobo_roles:" + ("G1 rot" if r1 else "G1 static") + "/" + ("G2 rot" if r2 else "G2 static")] += 1
        st["has_gobo1"] += "Gobo1" in gobo; st["has_gobo2"] += "Gobo2" in gobo
        st["colorrgb_and_coloradd"] += any(x.startswith("ColorRGB_") for x in gobo) and any(x.startswith("ColorAdd_") for x in gobo)
        st["colorrgb_only"] += any(x.startswith("ColorRGB_") for x in gobo) and not any(x.startswith("ColorAdd_") for x in gobo)
        st["null_attr"] += any(m["null_attrs"] for m in ms)
        st["has_16bit"] += any(m["bit16"] for m in ms)
        st["mixed_8_16_same_attr"] += len({(a, r) for m in ms for a, r in m["_res"].items() if a in ("Pan", "Tilt", "Dimmer")}) > \
            len({a for m in ms for a in m["_res"] if a in ("Pan", "Tilt", "Dimmer")})
        for m in ms:
            for g in m["instance_groups"]:
                ig_total += 1; ig_consistent += not g["odd_siblings"]
                fam_names[g["family"]] += 1
        per_fix.append({"id": fid, "name": f"{f['manufacturer']} {f['fixture']}", "modes": len(ms),
                        "max_channels": max(m["channels"] for m in ms),
                        "max_instances": max((g["count"] for m in ms for g in m["instance_groups"]), default=1)})
    R["broad"] = {"counts": dict(st), "modes": modes_n, "instance_groups": ig_total,
                  "instance_groups_name_consistent": ig_consistent,
                  "top_instance_family_names": fam_names.most_common(60),
                  "heaviest": sorted(per_fix, key=lambda x: -x["max_channels"])[:25],
                  "most_instances": sorted(per_fix, key=lambda x: -x["max_instances"])[:25]}
    # 5. part-kind rules on the heavy sample: every repeated part of each fixture's richest mode
    rows = []
    for fid, labs in LABELS.items():
        if fid not in heavy: continue
        f = heavy[fid]
        cand = [m for m in f["modes"] if m["instance_groups"]]
        m = max(cand, key=lambda m: (sum(g["count"] for g in m["instance_groups"]), m["channels"]))
        for g in m["instance_groups"]:
            lab = labs.get(g["family"])
            if lab is None: continue
            got = classify(f"{f['manufacturer']} {f['fixture']}", g)
            rows.append({"id": fid, "name": f["label"], "mode": m["name"], "family": g["family"], "count": g["count"],
                         "odd_siblings": g["odd_siblings"], "attrs": g["attrs"][:8], "label": lab, "rule": got,
                         "kind_hit": got[0] == lab[0],
                         "shape_scored": lab[1] is not None,
                         "shape_hit": lab[1] is not None and got[1] == lab[1]})
    ok = sum(r["kind_hit"] and (r["shape_hit"] or not r["shape_scored"]) for r in rows)
    R["parts"] = {"rows": rows, "groups": len(rows), "kind_hits": sum(r["kind_hit"] for r in rows),
                  "shape_scored": sum(r["shape_scored"] for r in rows),
                  "shape_hits": sum(r["shape_hit"] for r in rows), "both_hits": ok,
                  "fixtures_without_parts": [f"{heavy[k]['label']}" for k in heavy
                                             if not any(m["instance_groups"] for m in heavy[k]["modes"])]}
    # broad-sample rule coverage: how many instance groups get a confident shape
    cov = C(); seen = set(); nfix = 0
    for fid, f in broad.items():
        got_any = False
        for m in f["modes"]:
            for g in m["instance_groups"]:
                key = (fid, g["family"], g["count"])
                if key in seen: continue
                seen.add(key); got_any = True
                k, sh = classify(f"{f['manufacturer']} {f['fixture']}", g)
                cov[f"{k}/{sh}"] += 1
        nfix += got_any
    R["parts"]["broad_rule_output"] = dict(cov.most_common())
    R["parts"]["broad_fixtures_with_parts"] = nfix
    R["parts"]["broad_distinct_parts"] = len(seen)
    # compact: per fixture, issue counts over its ordered mode pairs + the first 3 pairs with issues
    R["mode_diffs"] = {fid: {"pairs": len(ds), "issue_pairs": dict(C(i for d in ds for i in d["issues"])),
                             "examples": [d for d in ds if d["issues"]][:3]}
                       for fid, ds in R["mode_diffs"].items()}
    for h in R["heavy"].values():
        for m in h["modes"]:
            m.pop("categories", None)
    json.dump(R, open(os.path.join(HERE, "sample.json"), "w"), indent=None, separators=(",", ":"), default=lambda o: sorted(o) if isinstance(o, set) else str(o))
    return R

if __name__ == "__main__":
    R = main()
    print(json.dumps({k: R[k] for k in ("mode_diff_totals",)}, indent=1))
    print(json.dumps(R["broad"]["counts"], indent=1), R["broad"]["modes"], R["broad"]["instance_groups"], R["broad"]["instance_groups_name_consistent"])
    print("parts", {k: v for k, v in R["parts"].items() if k not in ("rows",)})
    print("broad n", len([1 for k in R["broad"]["heaviest"]]), R["broad"]["top_instance_family_names"][:40])
    for r in R["parts"]["rows"]:
        if not r["kind_hit"] or (r["shape_scored"] and not r["shape_hit"]):
            print("  MISS", r["name"], r["family"], r["count"], "label", r["label"], "rule", r["rule"])
