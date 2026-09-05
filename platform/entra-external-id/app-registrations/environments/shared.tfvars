local_redirect_uri = "http://localhost:3000/"
# dev_redirect_uri intentionally omitted here — its default in variables.tf
# is now pinned to the real CloudFront domain (see ../CLAUDE.md). Override
# it with -var only if the distribution is ever replaced.
