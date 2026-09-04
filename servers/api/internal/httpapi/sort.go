package httpapi

import (
	"net/http"
	"strings"
)

// orderDirection reads the `order` query param (`asc`/`desc`,
// case-insensitive), defaulting to `desc` (newest first) for any unset or
// unrecognized value. Shared by every list endpoint.
func orderDirection(r *http.Request) string {
	if strings.EqualFold(r.URL.Query().Get("order"), "asc") {
		return "ASC"
	}
	return "DESC"
}
