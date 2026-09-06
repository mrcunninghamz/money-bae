package httpapi

import (
	"log/slog"
	"net/http"
	"time"
)

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (r *statusRecorder) WriteHeader(status int) {
	r.status = status
	r.ResponseWriter.WriteHeader(status)
}

func loggingMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		rec := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
		start := time.Now()
		next.ServeHTTP(rec, r)
		slog.Info("http request",
			"method", r.Method,
			"path", r.URL.Path,
			"status", rec.status,
			"duration", time.Since(start),
		)
	})
}

// internalError logs the real, underlying error at Error level — the
// per-request Info line above only ever sees the status code, not why a
// handler actually failed — then writes the same generic client-facing
// message and 500 status every caller already sent. Client response is
// unchanged; this only adds the log line needed to debug a 500 without
// having to reproduce it.
func internalError(w http.ResponseWriter, r *http.Request, clientMsg string, err error) {
	slog.Error("http request failed",
		"method", r.Method,
		"path", r.URL.Path,
		"error", err,
	)
	http.Error(w, clientMsg, http.StatusInternalServerError)
}
