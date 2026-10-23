import json, os, time, urllib.request, urllib.error, urllib.parse, hashlib, ssl
CTX = ssl.create_default_context(cafile="/etc/ssl/cert.pem")
BASE = "https://community.smidn.com"
RAW = os.path.join(os.path.dirname(os.path.abspath(__file__)), "raw")
os.makedirs(RAW, exist_ok=True)
_last = [0.0]
def _cache(key):
    return os.path.join(RAW, hashlib.sha1(key.encode()).hexdigest()[:16] + ".json")
def get(path, params=None, body=None):
    url = BASE + path + ("?" + urllib.parse.urlencode(params) if params else "")
    key = url + (json.dumps(body, sort_keys=True) if body else "")
    f = _cache(key)
    if os.path.exists(f):
        return json.load(open(f))
    for attempt in range(6):
        wait = float(os.environ.get("LC_API_GAP", "0.65")) - (time.time() - _last[0])
        if wait > 0: time.sleep(wait)
        _last[0] = time.time()
        req = urllib.request.Request(url, data=json.dumps(body).encode() if body else None,
              headers={"Content-Type": "application/json", "User-Agent": "lumacore-research"})
        try:
            with urllib.request.urlopen(req, timeout=60, context=CTX) as r:
                d = json.load(r)
            json.dump(d, open(f, "w"))
            open(os.path.join(RAW, "index.tsv"), "a").write(os.path.basename(f) + "\t" + key + "\n")
            return d
        except urllib.error.HTTPError as e:
            if e.code == 429:
                ra = float(e.headers.get("Retry-After") or 10); time.sleep(ra + 1); continue
            if e.code in (404, 400):
                d = {"ok": False, "status": e.code}
                json.dump(d, open(f, "w")); return d
            time.sleep(3)
        except Exception as e:
            time.sleep(3)
    return {"ok": False, "status": "failed"}
