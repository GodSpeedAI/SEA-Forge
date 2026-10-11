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


operator=BrowserSession(); oi=operator.login('operator')
rso=BrowserSession(); ri=rso.login('rso')
case_a,cur_a,ia=commit(operator,'T07_A',{'actor_id':'operator_local','role':'operator'})
case_b,cur_b,ib=commit(operator,'T07_B',{'actor_id':'rso_local','role':'R-SO'})
print('replay IDs',case_a,cur_a,ia,case_b,cur_b,ib)
op=snap(getworld(operator,case_a)); rs=snap(getworld(rso,case_a))
print('current',json.dumps({'operator_cursor':op.get('cursor'),'operator_prepare':(item(op,'task_prepare') or {}).get('status'),'operator_publish':(item(op,'task_publish') or {}).get('status'),'operator_actions':actions(op),'rso_cursor':rs.get('cursor'),'rso_prepare':(item(rs,'task_prepare') or {}).get('status'),'rso_actions':actions(rs)},sort_keys=True))
print('old',json.dumps({'operator':snap(getworld(operator,case_a,cur_a)),'rso':snap(getworld(rso,case_a,cur_a))},sort_keys=True))
