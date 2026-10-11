import http.client
import http.cookies
import json
import time
import urllib.error
import urllib.parse
import urllib.request

BASE = 'http://127.0.0.1:44280'
ORIGIN = 'http://127.0.0.1:4178'
DUMMY_PASSWORD = 'unused-dev-mode'

class Session:
    def __init__(self):
        self.cookies = {}
        self.csrf = None

    def request(self, method, path, body=None, extra=None):
        data = None if body is None else json.dumps(body, separators=(',', ':')).encode()
        headers = dict(extra or {})
        if data is not None:
            headers.setdefault('Content-Type', 'application/json')
        if self.cookies:
            headers['Cookie'] = '; '.join(f'{k}={v}' for k, v in self.cookies.items())
        req = urllib.request.Request(BASE + path, data=data, method=method, headers=headers)
        try:
            resp = urllib.request.urlopen(req, timeout=12)
        except urllib.error.HTTPError as exc:
            resp = exc
        with resp:
            for raw in resp.headers.get_all('Set-Cookie') or []:
                parsed = http.cookies.SimpleCookie()
                parsed.load(raw)
                for name, morsel in parsed.items():
                    self.cookies[name] = morsel.value
            return resp.status, resp.read()

    def login(self, username):
        status, _ = self.request('GET', '/api/session')
        assert status == 200, ('session bootstrap', status)
        pre = self.cookies.get('casework_csrf')
        assert pre
        status, body = self.request('POST', '/api/auth/login',
            {'username': username, 'password': DUMMY_PASSWORD},
            {'X-CSRF-Token': pre, 'Origin': ORIGIN})
        data = json.loads(body)
        assert status == 200 and data.get('authenticated') is True, ('login', status)
        self.csrf = data['csrf_token']

    def post_intent(self, body):
        return self.request('POST', '/api/intents', body,
            {'X-CSRF-Token': self.csrf, 'Origin': ORIGIN})


def json_body(response):
    status, raw = response
    return status, json.loads(raw)


def world(session, case=None, cursor=None):
    q = {}
    if case is not None:
        q['case_id'] = case
    if cursor is not None:
        q['cursor'] = cursor
    path = '/api/world' + (('?' + urllib.parse.urlencode(q)) if q else '')
    return session.request('GET', path)


def trajectory(session, case):
    path = '/api/trajectory?' + urllib.parse.urlencode({'case_id': case})
    status, body = json_body(session.request('GET', path))
    assert status == 200, ('trajectory', status)
    return [(p.get('cursor'), p.get('summary')) for p in body.get('points', [])]


def events_status(session, cursor):
    path = '/api/events?' + urllib.parse.urlencode({'last': cursor})
    conn = http.client.HTTPConnection('127.0.0.1', 44280, timeout=8)
    cookie = '; '.join(f'{k}={v}' for k, v in session.cookies.items())
    conn.request('GET', path, headers={'Cookie': cookie, 'Origin': ORIGIN, 'Accept': 'text/event-stream'})
    resp = conn.getresponse()
    status = resp.status
    if status == 200:
        prefix = []
        for _ in range(8):
            line = resp.readline()
            if not line or line in (b'\n', b'\r\n'):
                if prefix:
                    break
            else:
                prefix.append(line.decode('utf-8', 'replace').strip())
        result = {'status': status, 'event_prefix': prefix[:3]}
    else:
        try:
            payload = json.loads(resp.read(512))
            result = {'status': status, 'error': payload.get('error', {}).get('kind')}
        except Exception:
            result = {'status': status}
    conn.close()
    return result


def wait_ready():
    for _ in range(100):
        try:
            status, body = json_body(urllib_request('GET', '/api/readyz'))
            if status == 200:
                return body
        except Exception:
            pass
        time.sleep(0.1)
    raise AssertionError('gateway readiness did not return 200')


def urllib_request(method, path):
    req = urllib.request.Request(BASE + path, method=method)
    try:
        resp = urllib.request.urlopen(req, timeout=4)
    except urllib.error.HTTPError as exc:
        resp = exc
    with resp:
        return resp.status, resp.read()


wait_ready()
op = Session()
rso = Session()
op.login('operator')
rso.login('rso')
label = 't07-auth-replay-20260930'
preflight_status, preflight_raw = op.request('POST', '/api/templates/preflight',
    {'template_ref': 'e2e-sentry-chain@0.1.0', 'params': {'dataset_name': label}},
    {'X-CSRF-Token': op.csrf, 'Origin': ORIGIN})
preflight = json.loads(preflight_raw)
assert preflight_status == 200 and preflight.get('passed') is True and preflight.get('digest')
intent = {
    'intent_id': 't07-auth-replay-20260930-propose',
    'kind': 'CONSEQUENTIAL_CASE',
    'action_name': 'PROPOSE_CASE',
    'target_object_id': '',
    'case_id': '',
    'client_cursor': '',
    'actor': {'actor_id': 'rso_local', 'role': 'R-SO'},
    'parameters': {
        'template_ref': 'e2e-sentry-chain@0.1.0',
        'params': {'dataset_name': label},
        'preflight_digest': preflight['digest'],
    },
}
first_status, first_raw = op.post_intent(intent)
first = json.loads(first_raw)
assert first_status == 200 and first.get('success') is True, ('initial proposal', first_status)
case = first['resulting_object']['id']
receipt_cursor = first['new_cursor']
# Immediate receipt cursor reads establish that the accepted cursor was retained.
for session in (op, rso):
    status, raw = world(session, case, receipt_cursor)
    assert status == 200, ('immediate receipt history', status)
    assert json.loads(raw)['snapshot']['cursor'] == receipt_cursor
assert json.loads(world(op, case, receipt_cursor)[1])['snapshot']['perspective']['actor_id'] == 'operator_local'
baseline = trajectory(rso, case)
assert any(cursor == receipt_cursor for cursor, _ in baseline)
print(json.dumps({'phase': 'primed', 'case_id': case, 'intent_id': intent['intent_id'],
    'receipt_cursor': receipt_cursor, 'trajectory_points': len(baseline),
    'session_actor': 'operator_local', 'body_actor_was_spoofed': True}, sort_keys=True), flush=True)

assert input().strip() == 'REVOKED'
status, body = json_body(op.post_intent(intent))
assert status == 403 and body.get('error', {}).get('kind') == 'authority_denied', ('revoked replay', status, body.get('error', {}).get('kind'))
op_world_status, op_world_raw = world(op, case, receipt_cursor)
assert op_world_status == 403 and json.loads(op_world_raw).get('error', {}).get('kind') == 'authority_denied'
op_events = events_status(op, receipt_cursor)
assert op_events['status'] == 403 and op_events.get('error') == 'authority_denied'
rso_world_status, rso_world_raw = world(rso, case)
assert rso_world_status == 200
rso_events = events_status(rso, receipt_cursor)
assert rso_events['status'] == 200
revoked_trajectory = trajectory(rso, case)
assert revoked_trajectory == baseline, 'a revoked replay changed retained case history'
print(json.dumps({'phase': 'revoked', 'intent_status': status,
    'intent_error': body['error']['kind'], 'operator_world_status': op_world_status,
    'operator_events': op_events, 'rso_world_status': rso_world_status,
    'rso_events': rso_events, 'trajectory_unchanged': True,
    'trajectory_points': len(revoked_trajectory)}, sort_keys=True), flush=True)

assert input().strip() == 'RESTORED'
wait_ready()
replay_status, replay_raw = op.post_intent(intent)
assert replay_status == 200 and replay_raw == first_raw, ('restored exact receipt', replay_status)
replay = json.loads(replay_raw)
assert replay.get('success') is True and replay.get('new_cursor') == receipt_cursor
restored_world_status, restored_world_raw = world(op, case, receipt_cursor)
assert restored_world_status == 200 and json.loads(restored_world_raw)['snapshot']['cursor'] == receipt_cursor
restored_events = events_status(op, receipt_cursor)
assert restored_events['status'] == 200
restored_trajectory = trajectory(rso, case)
assert restored_trajectory == baseline, 'restored replay caused another kernel mutation'
print(json.dumps({'phase': 'restored', 'intent_status': replay_status,
    'exact_receipt_bytes_match': True, 'operator_world_status': restored_world_status,
    'operator_events': restored_events, 'trajectory_unchanged': True,
    'trajectory_points': len(restored_trajectory)}, sort_keys=True), flush=True)
