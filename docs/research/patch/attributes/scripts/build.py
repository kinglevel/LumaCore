import json,re,sys
D,OUT=sys.argv[1:3]
GSRC='GDTF 1.2 (DIN SPEC 15800:2022) mvrdevelopment/spec gdtf_attributes_with_description.json @098d379'
g=json.load(open(D+'/gdtf/gdtf_attributes_with_description.json'))['Attribute']
def rx(name):
    return re.compile('^'+re.escape(name).replace(r'\(n\)',r'(\d+)').replace(r'\(m\)',r'(\d+)')+'$',re.I)
pats=[(a['_name'],rx(a['_name'])) for a in g if '(n)' in a['_name'] or '(m)' in a['_name']]
def pattern_of(name):
    for p,r in pats:
        if r.match(name): return p
    return None
recs={}
for a in g:
    n=a['_name']; fg,f=a['_feature'].split('.',1)
    recs[n.lower()]=dict(name=n,pattern=('(n)' in n or '(m)' in n),feature_group=fg,feature=f,
        physical_unit=a.get('_physicalUnit'),main_attribute=a.get('_MainAttribute'),
        activation_group=a.get('_ActivationGroup'),pretty=a.get('_prettyName'),
        color_cie=a.get('_Color'),sub_physical_units=a.get('_subPhysicalUnits'),
        definition=a.get('definition') or None,
        source='GDTF 1.2',in_gdtf=True)
out=[]
for r in sorted(recs.values(),key=lambda r:(r['feature_group'],r['feature'],r['name'].lower())):
    out.append({k:v for k,v in r.items() if v is not None})
json.dump(out,open(OUT+'/attributes.json','w'),indent=1,ensure_ascii=False)
# feature groups
fgs={}
spec=open(D+'/gdtf/annexB.txt').read()
cur=None
for line in spec.splitlines():
    m1=re.search(r'<FeatureGroup\s+Name="([^"]+)"',line); m2=re.search(r'<Feature\s+Name="([^"]+)"',line)
    if m1: cur=m1.group(1); fgs[cur]=[]
    elif m2 and cur: fgs[cur].append(m2.group(1))
    if '</FeatureGroups>' in line: break
union={k:list(v) for k,v in fgs.items()}
# features actually used by attributes
for r in out:
    if r['feature'] not in union.setdefault(r['feature_group'],[]): union[r['feature_group']].append(r['feature'])
used={}
for r in out:
    used.setdefault(r['feature_group'],{}).setdefault(r['feature'],0); used[r['feature_group']][r['feature']]+=1
fgout=dict(union)
json.dump(fgout,open(OUT+'/feature_groups.json','w'),indent=1)
json.dump({'merged':fgout,'gdtf_1_2_annex_b':fgs,'attribute_count_per_feature':used,
  'gdtf_activation_groups':re.findall(r'<ActivationGroup\s+Name="([^"]+)"',spec)},
  open(OUT+'/feature_groups_detail.json','w'),indent=1)
# observed in GDTF Share
obs=[]
for c in json.load(open(D+'/smidn/api_attributes_all.json'))['facets']['gdtf']['categories']:
    for a in c['attributes']:
        n=a['name']; k=n.lower()
        hit=n if k in recs else pattern_of(n)
        obs.append(dict(name=n,count=a['count'],smidn_category=c['name'],gdtf_standard=hit))
obs.sort(key=lambda x:-x['count'])
json.dump(obs,open(OUT+'/observed_gdtf_share.json','w'),indent=0,ensure_ascii=False)
# stats
print('total',len(out))
print('patterns',sum(r['pattern'] for r in out))
print('fgs',len(fgout),'features',sum(len(v) for v in fgout.values()))
print(json.dumps(fgout))
tot=sum(o['count'] for o in obs); std=sum(o['count'] for o in obs if o['gdtf_standard'])
print('observed names',len(obs),'standard names',sum(1 for o in obs if o['gdtf_standard']),'channel-weighted pct',round(100*std/tot,1))
