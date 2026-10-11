import http.client, http.cookies, json, pathlib, time, urllib.error, urllib.parse, urllib.request
BASE='http://127.0.0.1:44179'; HOST='127.0.0.1'; PORT=44179; ORIGIN='http://127.0.0.1:4178'
LOG=pathlib.Path('/tmp/t07-final.uUPtaO/gateway-independent.log')
DUMMY_PASSWORD='whatever-dev-mode-skips'
class BrowserSession:
    def __init__(self): self.cookies={}; self.flags={}; self.csrf=None
    def req(self,method,path,body=None,headers=None):
        data=None if body is None else json.dumps(body).encode(); h=dict(headers or {})
        if data is not None: h.setdefault('Content-Type','application/json')
        if self.cookies: h['Cookie']='; '.join(f'{k}={v}' for k,v in self.cookies.items())
        request=urllib.request.Request(BASE+path,data=data,method=method,headers=h)
        try: response=urllib.request.urlopen(request,timeout=10)
        except urllib.error.HTTPError as e: response=e
        with response:
            out=dict(response.headers)
            for raw in response.headers.get_all('Set-Cookie') or []:
                parsed=http.cookies.SimpleCookie(); parsed.load(raw)
                for name,m in parsed.items():
                    self.cookies[name]=m.value
                    self.flags[name]={'secure':bool(m['secure']),'httponly':bool(m['httponly']),'samesite':m['samesite'] or None}
            return response.status,out,response.read()
    def login(self,user):
        s,_,_=self.req('GET','/api/session'); assert s==200
        pre=self.cookies.get('casework_csrf'); assert pre
        s,_,b=self.req('POST','/api/auth/login',{'username':user,'password':DUMMY_PASSWORD},{'X-CSRF-Token':pre,'Origin':ORIGIN})
        assert s==200,(s,b[:200]); d=json.loads(b); assert d.get('authenticated')
        self.csrf=d['csrf_token']; return d

def parse(response):
    s,h,b=response
    try: return s,h,json.loads(b)
    except Exception: return s,h,{'raw':b.decode(errors='replace')[:180]}
def getworld(sess,case=None,cursor=None,extras=None):
    q={}
    if case is not None:q['case_id']=case
    if cursor is not None:q['cursor']=cursor
    if extras:q.update(extras)
    return sess.req('GET','/api/world'+(('?'+urllib.parse.urlencode(q)) if q else ''))
def snap(response):
    s,h,d=parse(response); assert s==200,d; return d['snapshot']
def item(s,id): return next((o for o in s.get('visible_objects',[]) if o.get('id')==id),None)
def actions(s): return [a.get('intent') for a in s.get('available_actions',[])]
def post_intent(sess,id,action,case='',cursor='',params=None,actor=None,headers=None):
    body={'intent_id':id,'kind':'CONSEQUENTIAL_CASE','action_name':action,'target_object_id':'','case_id':case,'client_cursor':cursor,'actor':actor or {'actor_id':'operator_local','role':'operator'},'parameters':params or {}}
    h={'Origin':ORIGIN,'X-CSRF-Token':sess.csrf}; h.update(headers or {})
    return sess.req('POST','/api/intents',body,h)
def commit(sess,label,actor):
    params={'dataset_name':label}
    s,h,b=sess.req('POST','/api/templates/preflight',{'template_ref':'e2e-sentry-chain@0.1.0','params':params},{'Origin':ORIGIN,'X-CSRF-Token':sess.csrf})
    d=json.loads(b); assert s==200 and d.get('passed') and d.get('digest'),(s,d)
    intent_id='t07-'+label.lower().replace('_','-')+'-commit'
    resp=parse(post_intent(sess,intent_id,'PROPOSE_CASE',params={'template_ref':'e2e-sentry-chain@0.1.0','params':params,'preflight_digest':d['digest']},actor=actor))
    assert resp[0]==200 and resp[2].get('success') is True,resp
    payload=resp[2]; case=payload.get('resulting_object',{}).get('id'); assert case
    return case,payload.get('new_cursor'),intent_id

def stream_open(sess,last):
    conn=http.client.HTTPConnection(HOST,PORT,timeout=12)
    cookie='; '.join(f'{k}={v}' for k,v in sess.cookies.items())
    path='/api/events?last='+urllib.parse.quote(last,safe='')
    conn.request('GET',path,headers={'Cookie':cookie,'Origin':ORIGIN,'Accept':'text/event-stream'})
    response=conn.getresponse(); assert response.status==200, (response.status,response.read(200))
    return conn,response

def sse_event(response):
    name=''; data=[]
    while True:
        raw=response.readline()
        if not raw: return None,None
        line=raw.decode('utf-8','replace').rstrip('\r\n')
        if not line:
            if name or data: return name,json.loads('\n'.join(data))
            continue
        if line.startswith(':'): continue
        if line.startswith('event:'): name=line[6:].strip()
        elif line.startswith('data:'): data.append(line[5:].lstrip())
def next_case_snapshot(response,case,after=None,max_events=30):
    observed=[]
    for _ in range(max_events):
        event,data=sse_event(response)
        if event is None: break
        if event!='snapshot' or not isinstance(data,dict): continue
        p=data.get('payload',{})
        if p.get('case_id')==case and (after is None or p.get('cursor')!=after):
            return p,event,data
        observed.append((event,p.get('case_id'),p.get('cursor')))
    raise AssertionError(f'no matching SSE snapshot for case {case}; observed {observed}')

health=json.loads(urllib.request.urlopen(BASE+'/api/healthz',timeout=5).read()); ready=urllib.request.urlopen(BASE+'/api/readyz',timeout=8).status
assert health.get('provenance')=='go:live:sfwp' and ready==200,(health,ready)
# The direct gateway probes use only loopback; the preexisting Vite listener remains a trusted Origin.
operator=BrowserSession(); oi=operator.login('operator')
rso=BrowserSession(); ri=rso.login('rso')
unmapped=BrowserSession(); ui=unmapped.login('unmapped')
assert (oi['actor_id'],oi['role'])==('operator_local','operator')
assert (ri['actor_id'],ri['role'])==('rso_local','R-SO')
assert (ui['actor_id'],ui['role'])==('unmapped_local','operator')
assert operator.flags.get('casework_session')=={'secure':True,'httponly':True,'samesite':'Strict'}
assert operator.flags.get('casework_csrf')=={'secure':True,'httponly':False,'samesite':'Strict'}
# Anonymous protected reads refuse without a snapshot.
aw=parse(BrowserSession().req('GET','/api/world')); ae=parse(BrowserSession().req('GET','/api/events'))
assert aw[0]==401 and ae[0]==401 and 'snapshot' not in aw[2]
# Configured exact Origin is echoed; untrusted Origin receives no CORS authorization.
s,oh,_=operator.req('GET','/api/session',headers={'Origin':ORIGIN})
assert s==200 and oh.get('Access-Control-Allow-Origin')==ORIGIN
s,evil_headers,_=operator.req('GET','/api/world',headers={'Origin':'http://evil.invalid'})
assert s==200 and not evil_headers.get('Access-Control-Allow-Origin')
# Distinct current-world perspectives are derived from sessions.
op0=snap(getworld(operator)); rso0=snap(getworld(rso))
assert (op0['perspective']['actor_id'],op0['perspective']['role'])==('operator_local','operator')
assert (rso0['perspective']['actor_id'],rso0['perspective']['role'])==('rso_local','R-SO')
# Invalid delegation is verified before an invalid cached cursor lookup for both read surfaces.
for path in ('/api/world?cursor=cursor-that-does-not-exist','/api/events?last=cursor-that-does-not-exist'):
    s,_,d=parse(unmapped.req('GET',path)); assert s==403 and d.get('error',{}).get('kind')=='authority_denied',(path,s,d)
# Commit a real two-item governed case, then retain actor-specific old views.
case_a,_,intent_a=commit(operator,'T07_A',{'actor_id':'operator_local','role':'operator'})
op_a=snap(getworld(operator,case_a)); c_a=op_a['cursor']; assert c_a
rso_a=snap(getworld(rso,case_a)); assert rso_a['cursor']==c_a
hist_op_before=snap(getworld(operator,case_a,c_a)); hist_rso_before=snap(getworld(rso,case_a,c_a))
assert item(hist_op_before,'task_prepare') is not None and item(hist_rso_before,'task_prepare') is not None
assert 'EXECUTE_ITEM' in actions(hist_op_before) and 'EXECUTE_ITEM' not in actions(hist_rso_before)
# Overrides are rejected before looking up a valid retained cursor.
for path in (f'/api/world?cursor={urllib.parse.quote(c_a)}&actor=operator_local&role=operator',f'/api/events?last={urllib.parse.quote(c_a)}&actor=operator_local&role=operator'):
    s,_,d=parse(rso.req('GET',path)); assert s==400 and d.get('error',{}).get('kind')=='invalid' and 'identity' in d.get('error',{}).get('note','').lower(),(path,s,d)
# Commit a second case while claiming the R-SO in the request body from an operator session.
# Success and the resulting operator perspective prove the handler overwrites the claim.
case_b,_,intent_b=commit(operator,'T07_B',{'actor_id':'rso_local','role':'R-SO'})
op_b=snap(getworld(operator,case_b)); assert op_b['perspective']['actor_id']=='operator_local' and op_b['perspective']['role']=='operator'
c_b=op_b['cursor']; assert c_b
cross=parse(getworld(rso,case_b,c_a)); assert cross[0]==400 and cross[2].get('error',{}).get('kind')=='invalid'
# Replay retained revisions through SSE for each role before the next governed mutation.
replayed={}
for label,sess,identity in [('operator',operator,('operator_local','operator')),('rso',rso,('rso_local','R-SO'))]:
    conn,response=stream_open(sess,'')
    name,data=sse_event(response); assert name=='hello'
    rs,_,evt=next_case_snapshot(response,case_a)
    assert (rs['perspective']['actor_id'],rs['perspective']['role'])==identity
    assert ('EXECUTE_ITEM' in actions(rs))==(label=='operator')
    replayed[label]=rs['cursor']; conn.close()
# Open live streams after case B's cursor, then execute task_prepare in case A.
live={}
for label,sess in [('operator',operator),('rso',rso)]:
    conn,response=stream_open(sess,c_b); name,_=sse_event(response); assert name=='hello'; live[label]=(conn,response)
execute_id='t07-live-task-prepare-execute'
exec_response=parse(post_intent(operator,execute_id,'EXECUTE_ITEM',case_a,c_a,{'item_id':'task_prepare'}))
assert exec_response[0]==200 and exec_response[2].get('success') is True,exec_response
live_snaps={}
for label in ('operator','rso'):
    conn,response=live[label]
    p,_,_=next_case_snapshot(response,case_a,after=c_a)
    expected=('operator_local','operator') if label=='operator' else ('rso_local','R-SO')
    assert (p['perspective']['actor_id'],p['perspective']['role'])==expected
    assert ('EXECUTE_ITEM' in actions(p))==(label=='operator')
    live_snaps[label]=p
    conn.close()
current_op=snap(getworld(operator,case_a)); current_rso=snap(getworld(rso,case_a))
assert current_op['cursor']!=c_a
assert (current_op['perspective']['actor_id'],current_rso['perspective']['actor_id'])==('operator_local','rso_local')
# Historical snapshots remain byte-for-byte equal at the JSON value level after mutation.
hist_op_after=snap(getworld(operator,case_a,c_a)); hist_rso_after=snap(getworld(rso,case_a,c_a))
assert hist_op_before==hist_op_after and hist_rso_before==hist_rso_after
old_prepare=item(hist_op_after,'task_prepare'); new_prepare=item(current_op,'task_prepare')
assert old_prepare['status']!=new_prepare['status'],(old_prepare,new_prepare)
# The accepted first write unlocks the downstream sentry item for the operator only.
assert item(current_op,'task_publish') is not None
assert 'EXECUTE_ITEM' in actions(current_op) and 'EXECUTE_ITEM' not in actions(current_rso)
# Replayed SSE after the governed mutation delivers the same revision in each session's perspective.
replay_after={}
for label,sess,expected in [('operator',operator,('operator_local','operator')),('rso',rso,('rso_local','R-SO'))]:
    conn,response=stream_open(sess,c_b); name,_=sse_event(response); assert name=='hello'
    p,_,_=next_case_snapshot(response,case_a,after=c_a)
    assert (p['perspective']['actor_id'],p['perspective']['role'])==expected
    assert ('EXECUTE_ITEM' in actions(p))==(label=='operator')
    replay_after[label]=p['cursor']; conn.close()
# CSRF and untrusted-origin refusals do not move the kernel-backed projection.
pre_csrf_cursor=snap(getworld(operator,case_a))['cursor']
probe={'intent_id':'t07-no-csrf-write','kind':'CONSEQUENTIAL_CASE','action_name':'EXECUTE_ITEM','case_id':case_a,'client_cursor':pre_csrf_cursor,'parameters':{'item_id':'task_publish'}}
s,_,body=parse(operator.req('POST','/api/intents',probe,{'Origin':ORIGIN}))
assert s==403 and body.get('error',{}).get('kind')=='csrf_refused',(s,body)
probe['intent_id']='t07-untrusted-origin-write'
s,_,body=parse(operator.req('POST','/api/intents',probe,{'Origin':'http://evil.invalid','X-CSRF-Token':operator.csrf}))
assert s==403
assert snap(getworld(operator,case_a))['cursor']==pre_csrf_cursor
# Intent IDs become correlation IDs; a newline remains one quoted structured-log value.
log_id='t07-log-escape' + chr(10) + 'forged=true'
log_probe={'intent_id':log_id,'kind':'BAD','action_name':'NOT_A_REAL_ACTION','actor':{'actor_id':'rso_local','role':'R-SO'},'parameters':{}}
s,_,body=parse(operator.req('POST','/api/intents',log_probe,{'Origin':ORIGIN,'X-CSRF-Token':operator.csrf}))
assert s==200 and body.get('success') is False
lines=LOG.read_text(errors='replace').splitlines()
needle='correlation_id='+json.dumps(log_id,separators=(',',':'))
matching=[line for line in lines if needle in line]
assert len(matching)==1 and 'forged=true' not in [line for line in lines if line.strip()=='forged=true']
# The successful intent ID is used as the request-log correlation value.
assert any('correlation_id='+json.dumps(intent_a,separators=(',',':')) in line for line in lines)
print('auth/current: anonymous world/events=401; operator and R-SO identities/perspectives distinct; session cookie Secure+Strict, session HttpOnly')
print('origin/session: configured Origin accepted exactly; untrusted GET receives no CORS authorization and POST is refused; unmapped delegation world/events refused 403 before unknown cursor lookup')
print('case commits: operator case='+case_a+' cursor='+c_a+'; actor-overwrite case='+case_b+' perspective=operator; cross-case cursor=400 invalid')
print('actions: task_prepare EXECUTE_ITEM offered to operator and absent for R-SO; downstream task_publish unlocked for operator only')
print('history: task_prepare '+old_prepare['status']+' -> '+new_prepare['status']+' current; both actor-specific old snapshots exactly unchanged at '+c_a)
print('SSE: replay before mutation actor/action views pass; live and replay after mutation emit actor-specific snapshots for both roles')
print('writes/security: CSRF and untrusted-origin POSTs=403; current cursor unchanged; intent ID correlation logged; newline escaped in one log record')
print('SFWP provenance:',health.get('provenance'),'readiness_http:',ready,'execute_intent_id:',execute_id)
