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
case,_,_=commit(operator,'T07_C',{'actor_id':'operator_local','role':'operator'})
before=snap(getworld(operator,case)); before_cursor=before['cursor']
probe={'intent_id':'t07-no-csrf-write','kind':'CONSEQUENTIAL_CASE','action_name':'EXECUTE_ITEM','case_id':case,'client_cursor':before_cursor,'parameters':{'item_id':'task_publish'}}
s,_,b=parse(operator.req('POST','/api/intents',probe,{'Origin':ORIGIN}))
assert s==403 and b.get('error',{}).get('kind')=='csrf_refused',(s,b)
probe['intent_id']='t07-untrusted-origin-write'
s,_,b=parse(operator.req('POST','/api/intents',probe,{'Origin':'http://evil.invalid','X-CSRF-Token':operator.csrf}))
assert s==403 and b.get('error',{}).get('kind')=='csrf_refused',(s,b)
after=snap(getworld(operator,case)); assert after['cursor']==before_cursor
# Untrusted GET may be served, but without CORS permission for browser script access.
s,h,_=operator.req('GET','/api/session',headers={'Origin':'http://evil.invalid'})
assert s==200 and not h.get('Access-Control-Allow-Origin')
# Malformed intent ID newline is refused and logged as one escaped structured value.
log_id='t07-log-escape'+chr(10)+'forged=true'
probe={'intent_id':log_id,'kind':'BAD','action_name':'NOT_A_REAL_ACTION','actor':{'actor_id':'rso_local','role':'R-SO'},'parameters':{}}
s,_,b=parse(operator.req('POST','/api/intents',probe,{'Origin':ORIGIN,'X-CSRF-Token':operator.csrf}))
assert s==200 and b.get('success') is False,(s,b)
lines=LOG.read_text(errors='replace').splitlines()
needle='correlation_id='+json.dumps(log_id,separators=(',',':'))
matching=[line for line in lines if needle in line]
assert len(matching)==1 and not any(line.strip()=='forged=true' for line in lines)
assert any('correlation_id='+json.dumps('t07-t07-a-commit',separators=(',',':')) in line for line in lines)
print('write refusals: missing CSRF=403 csrf_refused; untrusted Origin POST=403 csrf_refused; current cursor unchanged',before_cursor)
print('origin: untrusted GET=200 without Access-Control-Allow-Origin; no browser CORS access')
print('log: newline intent ID refused; one escaped structured record, no forged second line; successful intent correlation ID present')
