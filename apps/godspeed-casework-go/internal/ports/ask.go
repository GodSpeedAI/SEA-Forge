package ports

import "context"

// AskQuestion is the semantic input to the governed self-disclosure service. Optional Case
// remains nil when the caller did not scope the question to a case.
type AskQuestion struct {
	Kind    string
	Subject string
	Purpose string
	Case    *CaseRef
}

// AskClaim is one grounded claim and its complete evidence and settlement references.
type AskClaim struct {
	ClaimID             string
	ClaimClass          string
	Subject             string
	Status              string
	Statement           string
	SnapshotRef         string
	EvidenceRefs        []string
	SettlementRefs      []string
	CapabilityRecordRef *string
}

// AskAnswer is the governed self-disclosure answer, including all claim references and metadata.
// A partial or denied disposition is still a successful answer, not an application error.
type AskAnswer struct {
	AnswerID            string
	QuestionID          string
	Disposition         string
	Claims              []AskClaim
	OmittedClaimClasses []string
	SnapshotRef         string
	Freshness           string
	Assurance           string
	Limitations         []string
	AuthorityNotice     string
	AnsweredAt          string
}

// AskPort asks the governed service a self-disclosure question for a verified effective actor.
type AskPort interface {
	Ask(ctx context.Context, actor ActorClaim, question AskQuestion) (AskAnswer, error)
}
