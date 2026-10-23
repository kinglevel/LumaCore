"""Stage 1: fetch everything into raw/ (cached). Run: python3 -I fetch.py"""
import sys, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import api

HEAVY = {  # id: short label
 "gdtf:40784":"MAC Aura XB","gdtf:159072":"MAC Aura XIP","gdtf:159073":"MAC Aura PXL",
 "gdtf:119729":"Robin Spiider","gdtf:119736":"Robin Tetra2","gdtf:119735":"Robin Tetra1",
 "gdtf:103537":"GLP JDC1","gdtf:150958":"GLP JDC2 IP V2","gdtf:159108":"Ayrton Diablo",
 "gdtf:162563":"GLP X4 Bar 20","gdtf:22993":"GLP X4 Bar 10","gdtf:150913":"Robin MegaPointe",
 "gdtf:146138":"MAC Viper Profile","gdtf:159118":"Ayrton Perseo Profile","gdtf:159992":"CP Sharpy Plus",
 "gdtf:135703":"Robin BMFL Spot","gdtf:46490":"GLP impression X4","gdtf:146713":"COLORado PXL Bar 16",
 "gdtf:89307":"COLORado PXL Bar 8","gdtf:56156":"Elation SixBar 1000","gdtf:144065":"Elation Chorus Line 16",
 "gdtf:149953":"Ayrton MagicPanel FX","gdtf:44146":"Elation Proteus Hybrid","gdtf:99663":"Robin Spikie",
 "gdtf:62991":"Astera AX1","gdtf:74459":"CP B-EYE K20","gdtf:119741":"Robin Tarrantula",
 "gdtf:149945":"Ayrton Khamsin","gdtf:116608":"Robin Esprite","gdtf:14093":"SGM P-5",
 "gdtf:162855":"Prolights HalupixDUO","gdtf:160435":"Ayrton Zonda 9 FX","gdtf:139321":"VDO Sceptron 10 1000",
 "gdtf:60186":"Robin 600 LEDWash","gdtf:78256":"Maverick Force 2 Profile","gdtf:91587":"Rogue R2X Wash",
 "gdtf:40094":"MAC Viper Wash","gdtf:143814":"LEDBeam 150 RGBW","gdtf:150111":"GLP impression X5",
 "gdtf:62324":"Robin Pointe","gdtf:36844":"MAC Encore Perf WRM","gdtf:144392":"CP Mythos",
 "gdtf:103011":"Martin Atomic 3000 LED","gdtf:113182":"Robin 1200 LEDWash","gdtf:89324":"Maverick MK3 Wash",
 "gdtf:37128":"MAC Quantum Wash","gdtf:74383":"Solaris Flare XL 120","gdtf:149939":"Ayrton Ghibli",
 "gdtf:142714":"COLORado PXL Curve 12","gdtf:147974":"COLORado PXL Curve 5","gdtf:160431":"Ayrton Argo 6 FX",
 "gdtf:147669":"Brighter X-DOT Max","gdtf:113709":"CP Tambora Stormy Linear",
 # revision pairs
 "gdtf:59461":"GLP X4 Bar 20 (older rev)","gdtf:145478":"Prolights HalupixDUO (older rev)",
}

def heavy():
    for fid in HEAVY:
        api.get(f"/api/fixtures/{fid}")
        api.get(f"/api/fixtures/{fid}/modes")
        api.get(f"/api/fixtures/{fid}/spectral")

def listing(sort, pages):
    rows = []
    for p in range(1, pages + 1):
        d = api.get("/api/fixtures", {"sort": sort, "order": "desc", "limit": 200, "page": p})
        rows += d.get("data", [])
    return rows

META = {}
def distinct(rows, n):
    seen, out = set(), []
    for r in rows:
        k = (r["manufacturer"].lower(), r["fixture"].lower())
        if k in seen or r["manufacturer"].lower().startswith("user test"): continue
        seen.add(k); out.append(r["id"]); META[r["id"]] = r
        if len(out) >= n: break
    return out

def broad():
    pick = {}
    big = os.environ.get("LC_BIG") == "1"
    plan = ((("rating", 8, 900), ("channels", 10, 600), ("date", 6, 500), ("name", 20, 1500)) if big
            else (("rating", 2, 160), ("channels", 4, 100), ("date", 1, 60)))
    for sort, pages, n in plan:
        for fid in distinct(listing(sort, pages), n):
            pick.setdefault(fid, sort)
    json.dump({k: {"via": v, **META[k]} for k, v in pick.items()},
              open(os.path.join(os.path.dirname(api.RAW), "broad_ids.json"), "w"), indent=0)
    from concurrent.futures import ThreadPoolExecutor
    with ThreadPoolExecutor(int(os.environ.get("LC_THREADS", "1"))) as ex:
        for i, _ in enumerate(ex.map(lambda f: api.get(f"/api/fixtures/{f}/modes"), list(pick))):
            if i % 200 == 0: print("broad", i, len(pick), flush=True)

if __name__ == "__main__":
    api.get("/api/attributes", {"sources": "gdtf"})
    api.get("/api/stats", {"sources": "gdtf"})
    heavy(); print("heavy done", flush=True)
    broad(); print("broad done", flush=True)
