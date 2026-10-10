package main

import (
	"net/http"
	"testing"
)

func TestNewHTTPServerCarriesSlowlorisBounds(t *testing.T) {
	srv := newHTTPServer("127.0.0.1:0", http.NewServeMux())
	if srv.ReadHeaderTimeout <= 0 || srv.IdleTimeout <= 0 || srv.MaxHeaderBytes <= 0 || srv.MaxHeaderBytes > 1<<20 {
		t.Fatalf("missing bounds: %+v", srv)
	}
	if srv.ReadTimeout != 0 || srv.WriteTimeout != 0 {
		t.Fatal("ReadTimeout/WriteTimeout would cut the SSE stream")
	}
}
