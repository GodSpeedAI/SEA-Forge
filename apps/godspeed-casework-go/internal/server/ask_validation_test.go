package server

import (
	"net/http"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestAskRejectsMalformedRawUTF8BeforeVerificationOrDispatch(t *testing.T) {
	cases := []struct {
		name string
		body []byte
	}{
		{
			name: "subject",
			body: append(append([]byte(`{"kind":"ask_capability","subject":"valid`), 0xff), []byte(`"}`)...),
		},
		{
			name: "purpose",
			body: append(append([]byte(`{"kind":"ask_capability","subject":"valid","purpose":"valid`), 0xff), []byte(`"}`)...),
		},
		{
			name: "case",
			body: append(append([]byte(`{"kind":"ask_capability","subject":"valid","case":"valid`), 0xff), []byte(`"}`)...),
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			h, fake := askFixture(t)
			op := h.login(t, "operator", "ignored-in-dev")
			perspectiveCalls := 0
			h.world.verify = func(_ ports.ActorClaim) error {
				perspectiveCalls++
				return nil
			}
			resp := postAskFrom(t, h, op, string(tc.body), "192.0.2.71", op.csrf, "")
			askStatus(t, resp, http.StatusBadRequest)
			actors, _ := fake.snapshot()
			if len(actors) != 0 {
				t.Fatalf("malformed UTF-8 reached AskPort %d times", len(actors))
			}
			if perspectiveCalls != 0 {
				t.Fatalf("malformed UTF-8 reached perspective verification %d times", perspectiveCalls)
			}
		})
	}
}
