package server

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"net/url"
	"reflect"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

type recordingAsk struct {
	mu        sync.Mutex
	actors    []ports.ActorClaim
	questions []ports.AskQuestion
	answer    ports.AskAnswer
	err       error
}

func (f *recordingAsk) Ask(_ context.Context, actor ports.ActorClaim, q ports.AskQuestion) (ports.AskAnswer, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.actors = append(f.actors, actor)
	f.questions = append(f.questions, q)
	return f.answer, f.err
}

func (f *recordingAsk) snapshot() ([]ports.ActorClaim, []ports.AskQuestion) {
	f.mu.Lock()
	defer f.mu.Unlock()
	return append([]ports.ActorClaim(nil), f.actors...), append([]ports.AskQuestion(nil), f.questions...)
}

func askFixture(t *testing.T) (*liveHarness, *recordingAsk) {
	t.Helper()
	h := newLiveHarness(t)
	fake := &recordingAsk{answer: sampleAskAnswer("answered")}
	h.api.opts.Ask = fake
	return h, fake
}

func sampleAskAnswer(disposition string) ports.AskAnswer {
	capRef := "capability-record-7"
	return ports.AskAnswer{
		AnswerID: "answer-9", QuestionID: "question-4", Disposition: disposition,
		Claims: []ports.AskClaim{{
			ClaimID: "claim-2", ClaimClass: "demonstrated_capability", Subject: "build",
			Status: "demonstrated", Statement: "The capability is demonstrated.", SnapshotRef: "snapshot-3",
			EvidenceRefs: []string{"evidence-8"}, SettlementRefs: []string{"settlement-6"},
			CapabilityRecordRef: &capRef,
		}}, OmittedClaimClasses: []string{"authority_requirements"}, SnapshotRef: "snapshot-3",
		Freshness: "current", Assurance: "local_tamper_evident", Limitations: []string{"bounded_view"},
		AuthorityNotice: "This answer grants no authority.", AnsweredAt: "2026-09-30T12:00:00Z",
	}
}

func askAnswerWire(answer ports.AskAnswer) map[string]any {
	claims := make([]any, 0, len(answer.Claims))
	for _, claim := range answer.Claims {
		wire := map[string]any{
			"claim_id": claim.ClaimID, "claim_class": claim.ClaimClass, "subject": claim.Subject,
			"status": claim.Status, "statement": claim.Statement, "snapshot_ref": claim.SnapshotRef,
			"evidence_refs": claim.EvidenceRefs, "settlement_refs": claim.SettlementRefs,
		}
		if claim.CapabilityRecordRef != nil {
			wire["capability_record_ref"] = *claim.CapabilityRecordRef
		}
		claims = append(claims, wire)
	}
	return map[string]any{
		"answer_id": answer.AnswerID, "question_id": answer.QuestionID, "disposition": answer.Disposition,
		"claims": claims, "omitted_claim_classes": answer.OmittedClaimClasses,
		"snapshot_ref": answer.SnapshotRef, "freshness": answer.Freshness, "assurance": answer.Assurance,
		"limitations": answer.Limitations, "authority_notice": answer.AuthorityNotice, "answered_at": answer.AnsweredAt,
	}
}

func postAskFrom(t *testing.T, h *liveHarness, u *testUser, body, remoteIP, csrf, forwarded string) *http.Response {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, u.base+"/api/ask", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	if csrf != "" {
		req.Header.Set(csrfHeader, csrf)
	}
	if forwarded != "" {
		req.Header.Set("X-Forwarded-For", forwarded)
	}
	req.RemoteAddr = net.JoinHostPort(remoteIP, "40123")
	base, err := url.Parse(u.base)
	if err != nil {
		t.Fatal(err)
	}
	for _, cookie := range u.client.Jar.Cookies(base) {
		req.AddCookie(cookie)
	}
	rr := httptest.NewRecorder()
	h.api.Handler().ServeHTTP(rr, req)
	return rr.Result()
}

func askBody(kind, subject string) string {
	return fmt.Sprintf(`{"kind":%q,"subject":%q}`, kind, subject)
}

func askStatus(t *testing.T, resp *http.Response, want int) []byte {
	t.Helper()
	defer resp.Body.Close()
	raw, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != want {
		t.Fatalf("POST /api/ask status=%d, want %d: %s", resp.StatusCode, want, raw)
	}
	return raw
}

func TestAskRequiresSessionThenCSRFBeforeCallingPort(t *testing.T) {
	h, fake := askFixture(t)
	resp, err := http.Post(h.ts.URL+"/api/ask", "application/json", strings.NewReader("not-json"))
	if err != nil {
		t.Fatal(err)
	}
	askStatus(t, resp, http.StatusUnauthorized)

	op := h.login(t, "operator", "ignored-in-dev")
	for _, token := range []string{"", "forged-token", ""} {
		resp := postAskFrom(t, h, op, askBody("ask_capability", "build"), "192.0.2.11", token, "203.0.113.1")
		askStatus(t, resp, http.StatusForbidden)
	}
	actors, _ := fake.snapshot()
	if len(actors) != 0 {
		t.Fatalf("session or CSRF rejection reached AskPort %d times", len(actors))
	}
	// The three CSRF refusals above must not consume the dedicated Ask session bucket.
	for i := 0; i < 2; i++ {
		resp := postAskFrom(t, h, op, askBody("ask_capability", "build"), "192.0.2.11", op.csrf, "")
		askStatus(t, resp, http.StatusOK)
	}
	resp = postAskFrom(t, h, op, askBody("ask_capability", "build"), "192.0.2.11", op.csrf, "")
	askStatus(t, resp, http.StatusTooManyRequests)
}

func TestAskUsesVerifiedSessionActorAndPreservesQuestionValues(t *testing.T) {
	h, fake := askFixture(t)
	op := h.login(t, "rso", "ignored-in-dev")
	body := `{"kind":"ask_environment_status","subject":"  deploy path  ","purpose":"","case":"  case_7  "}`
	resp := postAskFrom(t, h, op, body, "192.0.2.12", op.csrf, "")
	askStatus(t, resp, http.StatusOK)
	actors, questions := fake.snapshot()
	wantActor := ports.ActorClaim{ActorID: "rso_local", Role: "R-SO"}
	if len(actors) != 1 || actors[0] != wantActor {
		t.Fatalf("AskPort actor = %+v, want verified session actor %+v", actors, wantActor)
	}
	if len(questions) != 1 {
		t.Fatalf("AskPort calls=%d, want 1", len(questions))
	}
	q := questions[0]
	if q.Kind != "ask_environment_status" || q.Subject != "  deploy path  " || q.Purpose != "" || q.Case == nil || *q.Case != "  case_7  " {
		t.Fatalf("AskPort question changed supplied semantics: %+v", q)
	}
}

func TestAskAcceptsEveryApprovedKindAndDefaultsOnlyOmittedPurpose(t *testing.T) {
	h, fake := askFixture(t)
	kinds := []string{
		"ask_capability", "ask_operation_requirements", "ask_authority_requirements",
		"ask_projection_support", "ask_environment_status", "ask_failure_explanation",
		"ask_evidence_for_claim", "ask_available_affordances", "ask_why_denied",
	}
	for i, kind := range kinds {
		op := h.login(t, "operator", "ignored-in-dev")
		body := askBody(kind, "valid subject")
		if i == 1 {
			body = `{"kind":"` + kind + `","subject":"valid subject","purpose":""}`
		}
		resp := postAskFrom(t, h, op, body, fmt.Sprintf("198.51.100.%d", i+1), op.csrf, "")
		askStatus(t, resp, http.StatusOK)
	}
	_, questions := fake.snapshot()
	if len(questions) != len(kinds) {
		t.Fatalf("AskPort calls=%d, want %d", len(questions), len(kinds))
	}
	for i, q := range questions {
		if q.Kind != kinds[i] {
			t.Errorf("question %d kind=%q, want %q", i, q.Kind, kinds[i])
		}
		if i == 0 && q.Purpose != "planning" {
			t.Errorf("omitted purpose=%q, want planning", q.Purpose)
		}
		if i == 1 && q.Purpose != "" {
			t.Errorf("explicit empty purpose changed to %q", q.Purpose)
		}
		if q.Case != nil {
			t.Errorf("omitted case became present: %+v", q.Case)
		}
	}
}

func TestAskRejectsUnknownMalformedNullAndTrailingJSONWithoutCallingPort(t *testing.T) {
	h, fake := askFixture(t)
	cases := map[string]string{
		"unknown field":                    `{"kind":"ask_capability","subject":"s","actor":"forged"}`,
		"unknown actor id":                 `{"kind":"ask_capability","subject":"s","actor_id":"forged"}`,
		"unknown role":                     `{"kind":"ask_capability","subject":"s","role":"admin"}`,
		"null root":                        `null`,
		"array root":                       `[]`,
		"string root":                      `"ask"`,
		"number root":                      `7`,
		"boolean root":                     `true`,
		"null kind":                        `{"kind":null,"subject":"s"}`,
		"null subject":                     `{"kind":"ask_capability","subject":null}`,
		"null purpose":                     `{"kind":"ask_capability","subject":"s","purpose":null}`,
		"null case":                        `{"kind":"ask_capability","subject":"s","case":null}`,
		"wrong kind type":                  `{"kind":7,"subject":"s"}`,
		"wrong subject type":               `{"kind":"ask_capability","subject":[]}`,
		"wrong purpose type":               `{"kind":"ask_capability","subject":"s","purpose":false}`,
		"wrong case type":                  `{"kind":"ask_capability","subject":"s","case":{}}`,
		"malformed json":                   `{"kind":"ask_capability","subject":`,
		"second object":                    `{"kind":"ask_capability","subject":"s"}{}`,
		"garbage after object":             `{"kind":"ask_capability","subject":"s"}x`,
		"unknown kind":                     `{"kind":"ask_future","subject":"s"}`,
		"blank subject":                    `{"kind":"ask_capability","subject":" \t\n "}`,
		"blank case":                       `{"kind":"ask_capability","subject":"s","case":"  "}`,
		"purpose over 500 UTF-8 bytes":     `{"kind":"ask_capability","subject":"s","purpose":"` + strings.Repeat("a", 501) + `"}`,
		"multibyte purpose over 500 bytes": `{"kind":"ask_capability","subject":"s","purpose":"` + strings.Repeat("é", 251) + `"}`,
	}
	requestNo := 0
	for name, tc := range cases {
		requestNo++
		t.Run(name, func(t *testing.T) {
			op := h.login(t, "operator", "ignored-in-dev")
			resp := postAskFrom(t, h, op, tc, fmt.Sprintf("203.0.113.%d", requestNo), op.csrf, "")
			askStatus(t, resp, http.StatusBadRequest)
		})
	}
	actors, _ := fake.snapshot()
	if len(actors) != 0 {
		t.Fatalf("invalid bodies reached AskPort %d times", len(actors))
	}
}

func TestAskPreservesAllDispositionsAndDisclosureFields(t *testing.T) {
	h, fake := askFixture(t)
	for i, disposition := range []string{"answered", "partial", "denied"} {
		want := sampleAskAnswer(disposition)
		if disposition == "denied" {
			want.Claims[0].CapabilityRecordRef = nil
		}
		fake.answer = want
		op := h.login(t, "operator", "ignored-in-dev")
		resp := postAskFrom(t, h, op, askBody("ask_capability", "build"), fmt.Sprintf("192.0.2.%d", 20+i), op.csrf, "")
		raw := askStatus(t, resp, http.StatusOK)
		var got map[string]any
		if err := json.Unmarshal(raw, &got); err != nil {
			t.Fatalf("decode Ask answer: %v (%s)", err, raw)
		}
		wantBytes, err := json.Marshal(askAnswerWire(want))
		if err != nil {
			t.Fatal(err)
		}
		var wantWire map[string]any
		if err := json.Unmarshal(wantBytes, &wantWire); err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(got, wantWire) {
			t.Errorf("%s answer lost disclosure fields:\n got: %+v\nwant: %+v", disposition, got, wantWire)
		}
	}
}

func TestAskEnforcesUTF8PurposeAnd8192ByteRawBodyBoundaries(t *testing.T) {
	h, fake := askFixture(t)
	op := h.login(t, "operator", "ignored-in-dev")
	maxPurpose := strings.Repeat("é", 250) // exactly 500 UTF-8 bytes
	body := `{"kind":"ask_capability","subject":"s","purpose":"` + maxPurpose + `"}`
	resp := postAskFrom(t, h, op, body, "192.0.2.31", op.csrf, "")
	askStatus(t, resp, http.StatusOK)

	base := askBody("ask_capability", "s")
	if len(base) > 8192 {
		t.Fatal("test body unexpectedly exceeds raw byte limit")
	}
	maxBody := base + strings.Repeat(" ", 8192-len(base))
	op = h.login(t, "operator", "ignored-in-dev")
	resp = postAskFrom(t, h, op, maxBody, "192.0.2.32", op.csrf, "")
	askStatus(t, resp, http.StatusOK)
	tooLarge := maxBody + " "
	op = h.login(t, "operator", "ignored-in-dev")
	resp = postAskFrom(t, h, op, tooLarge, "192.0.2.33", op.csrf, "")
	askStatus(t, resp, http.StatusRequestEntityTooLarge)
	_, questions := fake.snapshot()
	if len(questions) != 2 {
		t.Fatalf("AskPort calls=%d, want 2 successful requests", len(questions))
	}
	if questions[0].Purpose != maxPurpose {
		t.Fatal("500-byte multibyte purpose changed before AskPort")
	}
}

func TestAskPerspectiveMustBeVerifiedBeforeDispatch(t *testing.T) {
	h, fake := askFixture(t)
	op := h.login(t, "rso", "ignored-in-dev")
	h.world.verify = func(actor ports.ActorClaim) error {
		if actor != (ports.ActorClaim{ActorID: "rso_local", Role: "R-SO"}) {
			t.Errorf("perspective verification actor=%+v", actor)
		}
		return apperr.New(apperr.KindAuthorityDenied, "", "identity", "revoked")
	}
	resp := postAskFrom(t, h, op, askBody("ask_capability", "build"), "192.0.2.41", op.csrf, "")
	askStatus(t, resp, http.StatusForbidden)
	actors, _ := fake.snapshot()
	if len(actors) != 0 {
		t.Fatalf("refused perspective reached AskPort %d times", len(actors))
	}
}

func TestAskLimiterUsesRealIPAndIsSeparateFromSessionAndIntentBuckets(t *testing.T) {
	h, fake := askFixture(t)
	// Two asks exhaust only this session bucket; the third is refused before the port.
	a := h.login(t, "operator", "ignored-in-dev")
	for i := 0; i < 2; i++ {
		resp := postAskFrom(t, h, a, askBody("ask_capability", "s"), "198.51.100.50", a.csrf, "198.51.100.1")
		askStatus(t, resp, http.StatusOK)
	}
	resp := postAskFrom(t, h, a, askBody("ask_capability", "s"), "198.51.100.50", a.csrf, "198.51.100.2")
	askStatus(t, resp, http.StatusTooManyRequests)

	// A separate session shares the same remote IP budget and still has its own burst.
	b := h.login(t, "rso", "ignored-in-dev")
	resp = postAskFrom(t, h, b, askBody("ask_capability", "s"), "198.51.100.50", b.csrf, "203.0.113.8")
	askStatus(t, resp, http.StatusOK)

	// Four different sessions can spend the IP burst; changing X-Forwarded-For does not
	// manufacture new IP buckets.
	for i := 0; i < 7; i++ {
		u := h.login(t, "operator", "ignored-in-dev")
		resp = postAskFrom(t, h, u, askBody("ask_capability", "s"), "203.0.113.50", u.csrf,
			fmt.Sprintf("198.51.100.%d", i+20))
		if i < 4 {
			askStatus(t, resp, http.StatusOK)
		} else {
			askStatus(t, resp, http.StatusTooManyRequests)
		}
	}

	actors, _ := fake.snapshot()
	if len(actors) != 7 { // 3 admitted on the first IP plus 4 on the second IP
		t.Fatalf("rate-limited requests reached AskPort: calls=%d, want 7", len(actors))
	}
	if h.api.intentsLimiter.Len() != 0 {
		t.Fatalf("Ask traffic used the intent limiter buckets: %d", h.api.intentsLimiter.Len())
	}
}

func TestAskSessionLimiterRefillsAndIntentLimiterRemainsUsable(t *testing.T) {
	h, fake := askFixture(t)
	op := h.login(t, "operator", "ignored-in-dev")
	for i := 0; i < 2; i++ {
		resp := postAskFrom(t, h, op, askBody("ask_capability", "s"), "192.0.2.60", op.csrf, "")
		askStatus(t, resp, http.StatusOK)
	}
	// At six per minute, five seconds returns only half a token and must still be denied.
	time.Sleep(5100 * time.Millisecond)
	resp := postAskFrom(t, h, op, askBody("ask_capability", "s"), "192.0.2.60", op.csrf, "")
	askStatus(t, resp, http.StatusTooManyRequests)
	// The denied attempt retains that half token; another five seconds makes one token available.
	time.Sleep(5100 * time.Millisecond)
	resp = postAskFrom(t, h, op, askBody("ask_capability", "s"), "192.0.2.60", op.csrf, "")
	askStatus(t, resp, http.StatusOK)
	if h.api.intentsLimiter.Len() != 0 {
		t.Fatalf("Ask calls changed the existing intent limiter: %d buckets", h.api.intentsLimiter.Len())
	}
	for i := 0; i < 3; i++ {
		resp, err := op.post("/api/intents", "application/json", validIntentBody(fmt.Sprintf("ask-separate-intent-%d", i)))
		if err != nil {
			t.Fatal(err)
		}
		askStatus(t, resp, http.StatusOK)
	}
	if got := h.ints.dispatched(); got != 3 {
		t.Fatalf("existing intent route dispatched %d requests after Ask traffic, want 3", got)
	}
	actors, _ := fake.snapshot()
	if len(actors) != 3 {
		t.Fatalf("AskPort calls=%d, want 3 including refilled request", len(actors))
	}
}

func TestAskTrailingWhitespaceIsAllowedWithinRawLimit(t *testing.T) {
	h, fake := askFixture(t)
	op := h.login(t, "operator", "ignored-in-dev")
	body := bytes.TrimSpace([]byte(askBody("ask_capability", "s")))
	body = append(body, []byte(" \n\t ")...)
	resp := postAskFrom(t, h, op, string(body), "192.0.2.70", op.csrf, "")
	askStatus(t, resp, http.StatusOK)
	actors, _ := fake.snapshot()
	if len(actors) != 1 {
		t.Fatalf("trailing whitespace request made %d Ask calls, want 1", len(actors))
	}
}
