import http.client, http.cookies, json, pathlib, time, urllib.error, urllib.parse, urllib.request
BASE='http://127.0.0.1:44279'; HOST='127.0.0.1'; PORT=44279; ORIGIN='http://127.0.0.1:4178'
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

# Fresh fixed-binary green live probe. The first historical read directly consumes each accepted new_cursor.
health=json.loads(urllib.request.urlopen(BASE+'/api/healthz',timeout=5).read())
ready=urllib.request.urlopen(BASE+'/api/readyz',timeout=8).status
assert health.get('provenance')=='go:live:sfwp' and ready==200,(health,ready)
operator=BrowserSession(); oi=operator.login('operator')
rso=BrowserSession(); ri=rso.login('rso')
unmapped=BrowserSession(); ui=unmapped.login('unmapped')
assert (oi['actor_id'],oi['role'])==('operator_local','operator')
assert (ri['actor_id'],ri['role'])==('rso_local','R-SO')
assert (ui['actor_id'],ui['role'])==('unmapped_local','operator')
assert operator.flags.get('casework_session')=={'secure':True,'httponly':True,'samesite':'Strict'}
assert operator.flags.get('casework_csrf')=={'secure':True,'httponly':False,'samesite':'Strict'}
aw=parse(BrowserSession().req('GET','/api/world')); ae=parse(BrowserSession().req('GET','/api/events'))
assert aw[0]==401 and ae[0]==401 and 'snapshot' not in aw[2]
s,oh,_=operator.req('GET','/api/session',headers={'Origin':ORIGIN})
assert s==200 and oh.get('Access-Control-Allow-Origin')==ORIGIN
s,eh,_=operator.req('GET','/api/world',headers={'Origin':'http://evil.invalid'})
assert s==200 and not eh.get('Access-Control-Allow-Origin')
for path in ('/api/world?cursor=cursor-that-does-not-exist','/api/events?last=cursor-that-does-not-exist'):
    s,_,d=parse(unmapped.req('GET',path)); assert s==403 and d.get('error',{}).get('kind')=='authority_denied',(path,s,d)
# First response cursor proof: no unrelated gateway call between PROPOSE_CASE response and both history reads.
case_a,c_a,intent_a=commit(operator,'T07_GREEN_A',{'actor_id':'operator_local','role':'operator'})
first_op=getworld(operator,case_a,c_a)
first_rs=getworld(rso,case_a,c_a)
op_a=snap(first_op); rs_a=snap(first_rs)
assert op_a['cursor']==c_a and rs_a['cursor']==c_a
assert (op_a['perspective']['actor_id'],op_a['perspective']['role'])==('operator_local','operator')
assert (rs_a['perspective']['actor_id'],rs_a['perspective']['role'])==('rso_local','R-SO')
assert 'EXECUTE_ITEM' in [a.get('intent') for a in item(op_a,'task_prepare').get('actions',[])]
assert not item(rs_a,'task_prepare').get('actions',[])
assert item(op_a,'task_publish')['status']=='WAITING'
trajectory_status,_,trajectory_body=operator.req('GET','/api/trajectory?case_id='+urllib.parse.quote(case_a,safe=''))
trajectory=json.loads(trajectory_body); point_cursors=[p.get('cursor') for p in trajectory.get('points',[])]
assert trajectory_status==200 and c_a in point_cursors,(trajectory_status,trajectory)
# Overrides must refuse before history; cross-case cursor must be typed invalid.
for path in (f'/api/world?cursor={urllib.parse.quote(c_a)}&actor=operator_local&role=operator',f'/api/events?last={urllib.parse.quote(c_a)}&actor=operator_local&role=operator'):
    s,_,d=parse(rso.req('GET',path)); assert s==400 and d.get('error',{}).get('kind')=='invalid' and 'identity' in d.get('error',{}).get('note','').lower(),(path,s,d)
# A forged R-SO claim in an operator session is overwritten by session identity.
case_b,c_b,intent_b=commit(operator,'T07_GREEN_B',{'actor_id':'rso_local','role':'R-SO'})
op_b=snap(getworld(operator,case_b,c_b))
assert op_b['perspective']['actor_id']=='operator_local' and op_b['perspective']['role']=='operator'
cross=parse(getworld(rso,case_b,c_a)); assert cross[0]==400 and cross[2].get('error',{}).get('kind')=='invalid'
# Replay sessions before mutation and verify actor-specific actions.
replayed={}
for label,sess,identity in [('operator',operator,('operator_local','operator')),('rso',rso,('rso_local','R-SO'))]:
    conn,response=stream_open(sess,''); name,_=sse_event(response); assert name=='hello'
    p,_,_=next_case_snapshot(response,case_a)
    assert (p['perspective']['actor_id'],p['perspective']['role'])==identity
    task=item(p,'task_prepare')
    assert ('EXECUTE_ITEM' in [a.get('intent') for a in task.get('actions',[])])==(label=='operator')
    replayed[label]=p['cursor']; conn.close()
# Preserve response bytes before mutation, open live streams at exact current case cursor.
before_op=getworld(operator,case_a,c_a); before_rs=getworld(rso,case_a,c_a)
live={}
for label,sess in [('operator',operator),('rso',rso)]:
    conn,response=stream_open(sess,c_a); name,_=sse_event(response); assert name=='hello'; live[label]=(conn,response)
execute_id='t07-green-prepare-execute'
exec_result=parse(post_intent(operator,execute_id,'EXECUTE_ITEM',case_a,c_a,{'item_id':'task_prepare'}))
assert exec_result[0]==200 and exec_result[2].get('success') is True,exec_result
execute_cursor=exec_result[2].get('new_cursor'); assert execute_cursor and execute_cursor>c_a,exec_result
seen={}
for label in ('operator','rso'):
    conn,response=live[label]; frames=[]
    for _ in range(50):
        name,data=sse_event(response)
        if name is None: break
        if name!='snapshot' or not isinstance(data,dict): continue
        p=data.get('payload',{}); ss=p.get('snapshot',p)
        if p.get('case_id')!=case_a: continue
        prepare=item(ss,'task_prepare'); publish=item(ss,'task_publish')
        ident=(ss.get('perspective',{}).get('actor_id'),ss.get('perspective',{}).get('role'))
        prepare_actions=[a.get('intent') for a in (prepare or {}).get('actions',[])]
        publish_actions=[a.get('intent') for a in (publish or {}).get('actions',[])]
        frames.append({'cursor':p.get('cursor'),'identity':ident,'prepare':(prepare or {}).get('status'),'prepare_actions':prepare_actions,'publish':(publish or {}).get('status'),'publish_actions':publish_actions})
        if (prepare or {}).get('status')=='COMPLETED' and (publish or {}).get('status')=='READY_TO_BEGIN': break
    seen[label]=frames; conn.close()
    expected=('operator_local','operator') if label=='operator' else ('rso_local','R-SO')
    assert frames and frames[-1]['identity']==expected and frames[-1]['prepare']=='COMPLETED' and frames[-1]['publish']=='READY_TO_BEGIN',(label,frames)
    if label=='operator': assert 'EXECUTE_ITEM' in frames[-1]['publish_actions']
    else: assert not frames[-1]['prepare_actions'] and not frames[-1]['publish_actions']
# Immediate retained old snapshots remain byte-identical after governed mutation.
after_op=getworld(operator,case_a,c_a); after_rs=getworld(rso,case_a,c_a)
assert after_op[0]==200 and after_rs[0]==200
assert before_op[2]==after_op[2] and before_rs[2]==after_rs[2]
old_op=snap(after_op); old_rs=snap(after_rs)
assert item(old_op,'task_prepare')['status']=='READY_TO_BEGIN' and item(old_op,'task_publish')['status']=='WAITING'
assert item(old_rs,'task_prepare')['status']=='READY_TO_BEGIN'
# Latest current state may include later terminal frames after intent new_cursor; wait for state, not equality.
for _ in range(80):
    now_op=snap(getworld(operator,case_a)); now_rs=snap(getworld(rso,case_a))
    if item(now_op,'task_prepare')['status']=='COMPLETED' and item(now_op,'task_publish')['status']=='READY_TO_BEGIN': break
    time.sleep(.1)
assert now_op['cursor']>=execute_cursor and now_rs['cursor']>=execute_cursor
assert item(now_op,'task_publish')['status']=='READY_TO_BEGIN'
assert 'EXECUTE_ITEM' in [a.get('intent') for a in item(now_op,'task_publish').get('actions',[])]
assert not item(now_rs,'task_publish').get('actions',[])
# CSRF / Origin no-write checks after mutation.
before_refusal=now_op['cursor']
probe={'intent_id':'t07-green-no-csrf','kind':'CONSEQUENTIAL_CASE','action_name':'EXECUTE_ITEM','case_id':case_a,'client_cursor':before_refusal,'parameters':{'item_id':'task_publish'}}
s,_,b=parse(operator.req('POST','/api/intents',probe,{'Origin':ORIGIN}))
assert s==403 and b.get('error',{}).get('kind')=='csrf_refused',(s,b)
probe['intent_id']='t07-green-untrusted-origin'
s,_,b=parse(operator.req('POST','/api/intents',probe,{'Origin':'http://evil.invalid','X-CSRF-Token':operator.csrf}))
assert s==403 and b.get('error',{}).get('kind')=='csrf_refused',(s,b)
assert snap(getworld(operator,case_a))['cursor']==before_refusal
# Escaped log value and successful mutation correlation, observed in gateway's process stdout.
log_id='t07-green-log-escape'+chr(10)+'forged=true'
log_probe={'intent_id':log_id,'kind':'BAD','action_name':'NOT_A_REAL_ACTION','actor':{'actor_id':'rso_local','role':'R-SO'},'parameters':{}}
s,_,b=parse(operator.req('POST','/api/intents',log_probe,{'Origin':ORIGIN,'X-CSRF-Token':operator.csrf}))
assert s==200 and b.get('success') is False,(s,b)
print('health/readiness',health,ready,'roles',oi['actor_id'],oi['role'],ri['actor_id'],ri['role'],'cookie_flags',operator.flags)
print('anonymous',aw[0],ae[0],'unmapped unknown cursor',403,'trusted/untrusted CORS',oh.get('Access-Control-Allow-Origin'),eh.get('Access-Control-Allow-Origin'))
print('case_a immediate accepted cursor',case_a,c_a,'world statuses',first_op[0],first_rs[0],'trajectory',trajectory.get('base_cursor'),trajectory.get('head_cursor'),len(point_cursors))
print('case_b actor overwrite',case_b,op_b['perspective'],'crosscase',cross[0])
print('replay cursors',replayed,'live frames',json.dumps(seen,sort_keys=True))
print('immutable old HTTP bodies',len(before_op[2]),len(before_rs[2]),'byte_identical',before_op[2]==after_op[2] and before_rs[2]==after_rs[2])
print('execution response cursor',execute_cursor,'current cursor',now_op['cursor'],'state',item(now_op,'task_prepare')['status'],item(now_op,'task_publish')['status'])
print('CSRF/origin refusals',403,403,'cursor_unchanged',before_refusal,'newline invalid intent status',s,'success',b.get('success'))
print('successful intent IDs',intent_a,intent_b,execute_id,'newline log intent ID retained only in gateway stdout')

