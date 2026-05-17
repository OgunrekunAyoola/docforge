#!/bin/sh
set -e

echo "Starting docforge..."

# Run migrations before starting (idempotent - safe to run every boot)
echo "Running database migrations..."
docforge --migrate-only 2>/dev/null || true

exec docforge "$@"
