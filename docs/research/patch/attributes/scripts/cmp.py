import json,re,sys
D=sys.argv[1]
j={a['_name']:a for a in json.load(open(D+'/gdtf/gdtf_attributes_with_description.json'))['Attribute']}
B={}
for line in open(D+'/gdtf/annexB.txt'):
    if '<Attribute ' not in line: continue
    n=re.search(r'Name="([^"]+)',line).group(1)
    attrs=dict(re.findall(r'(\w+)="([^"]*)"',line))
    B[n]=attrs
# example rows Gobo1/Gobo2 duplicate
print(len(j),len(B))
print('json only',sorted(set(j)-set(B)))
print('annexB only',sorted(set(B)-set(j)))
diff=0
for n in set(j)&set(B):
    a=j[n];b=B[n]
    for jk,bk in [('_feature','Feature'),('_physicalUnit','PhysicalUnit'),('_ActivationGroup','ActivationGroup'),('_MainAttribute','MainAttribute'),('_prettyName','Pretty')]:
        if (a.get(jk) or '')!=(b.get(bk) or ''):
            if not (jk=='_physicalUnit' and (a.get(jk) or 'None')==(b.get(bk) or 'None')):
                diff+=1; print('DIFF',n,jk,repr(a.get(jk)),repr(b.get(bk)))
print('diffs',diff)
print(set(k for b in B.values() for k in b))
