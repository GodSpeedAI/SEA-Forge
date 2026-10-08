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


operator=BrowserSession(); operator.login('operator')
rso=BrowserSession(); rso.login('rso')
case,commit_cursor,_=commit(operator,'T07_C',{'actor_id':'operator_local','role':'operator'})
initial_cursor=snap(getworld(operator,case)).get('cursor')
assert initial_cursor
pre_op=getworld(operator,case,initial_cursor); pre_rs=getworld(rso,case,initial_cursor)
pre_op_snap=snap(pre_op); pre_rs_snap=snap(pre_rs)
assert item(pre_op_snap,'task_prepare')['status']=='READY_TO_BEGIN'
assert 'EXECUTE_ITEM' in [a.get('intent') for a in item(pre_op_snap,'task_prepare').get('actions',[])]
assert not item(pre_rs_snap,'task_prepare').get('actions',[])
streams={}
for label,sess in [('operator',operator),('rso',rso)]:
    conn,response=stream_open(sess,initial_cursor); name,_=sse_event(response); assert name=='hello'; streams[label]=(conn,response)
intent='t07-case-c-task-prepare-execute'
resp=parse(post_intent(operator,intent,'EXECUTE_ITEM',case,initial_cursor,{'item_id':'task_prepare'}))
assert resp[0]==200 and resp[2].get('success') is True,resp
response_cursor=resp[2].get('new_cursor'); assert response_cursor and response_cursor!=initial_cursor,resp[2]
seen={}
for label in ('operator','rso'):
    conn,response=streams[label]; frames=[]
    for _ in range(50):
        name,data=sse_event(response)
        if name is None: break
        if name!='snapshot' or not isinstance(data,dict): continue
        p=data.get('payload',{})
        if p.get('case_id')!=case: continue
        ss=p.get('snapshot',p); task=item(ss,'task_prepare')
        ident=(ss.get('perspective',{}).get('actor_id'),ss.get('perspective',{}).get('role'))
        item_actions=[a.get('intent') for a in (task or {}).get('actions',[])]
        frames.append({'cursor':p.get('cursor'),'identity':ident,'status':(task or {}).get('status'),'task_prepare_actions':item_actions})
        if (task or {}).get('status')=='COMPLETED' and p.get('cursor')>=response_cursor: break
    seen[label]=frames; conn.close()
for label in ('operator','rso'):
    expected=('operator_local','operator') if label=='operator' else ('rso_local','R-SO')
    assert seen[label] and seen[label][-1]['identity']==expected,(label,seen[label])
    if label=='operator': assert all(('EXECUTE_ITEM' in f['task_prepare_actions'])==(f['status']=='READY_TO_BEGIN') for f in seen[label]),seen[label]
    else: assert all('EXECUTE_ITEM' not in f['task_prepare_actions'] for f in seen[label]),seen[label]
# Wait for the latest projection to settle, allowing a later terminal revision beyond response_cursor.
for _ in range(80):
    now_op=snap(getworld(operator,case)); now_rs=snap(getworld(rso,case))
    if item(now_op,'task_prepare')['status']=='COMPLETED' and item(now_rs,'task_prepare')['status']=='COMPLETED': break
    time.sleep(.1)
assert item(now_op,'task_prepare')['status']=='COMPLETED' and item(now_rs,'task_prepare')['status']=='COMPLETED'
after_op=getworld(operator,case,initial_cursor); after_rs=getworld(rso,case,initial_cursor)
assert pre_op[2]==after_op[2] and pre_rs[2]==after_rs[2]
old_op=snap(after_op); old_rs=snap(after_rs)
assert old_op==pre_op_snap and old_rs==pre_rs_snap
assert item(old_op,'task_prepare')['status']=='READY_TO_BEGIN' and item(now_op,'task_prepare')['status']=='COMPLETED'
assert 'EXECUTE_ITEM' in [a.get('intent') for a in item(now_op,'task_publish').get('actions',[])]
assert not item(now_rs,'task_publish').get('actions',[])
print('case',case,'old',initial_cursor,'mutation response cursor',response_cursor,'latest',now_op['cursor'])
print('old API bodies byte-identical across governed mutation',len(pre_op[2]),len(pre_rs[2]),'operator/R-SO')
print('state-aware SSE',json.dumps(seen,sort_keys=True))
print('current task_prepare=COMPLETED; task_publish=READY_TO_BEGIN offered only to operator at item-level')
