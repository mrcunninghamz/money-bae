FROM golang:1.27.1-alpine AS builder
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
# App Runner only runs x86_64, so target amd64 explicitly rather than
# whatever architecture happens to be building the image (e.g. arm64 on
# Apple Silicon).
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /app/server ./cmd/api

FROM --platform=linux/amd64 gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /app/server /server
ENTRYPOINT ["/server"]
