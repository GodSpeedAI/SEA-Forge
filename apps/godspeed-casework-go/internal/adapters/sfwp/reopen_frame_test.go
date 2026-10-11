package sfwp

import "testing"

func TestNewCaseReopenCarriesANonBlankReasonAndOmitsABlankOne(t *testing.T) {
	var g Governance
	with, err := NewCaseReopen("case_1", "late evidence arrived", "", "req-1", g)
	if err != nil {
		t.Fatal(err)
	}
	if with.body["reason"] != "late evidence arrived" || with.body["case_id"] != "case_1" {
		t.Fatalf("body = %+v", with.body)
	}
	for _, blank := range []string{"", "   "} {
		without, err := NewCaseReopen("case_1", blank, "", "req-2", g)
		if err != nil {
			t.Fatal(err)
		}
		if _, ok := without.body["reason"]; ok {
			t.Fatalf("a blank reason %q must not be sent: %+v", blank, without.body)
		}
	}
}
