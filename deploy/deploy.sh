#!/bin/sh
# Builds the site and deploys it to Cloud Run, behind a username + password.
#   deploy/deploy.sh [gcp-project] [region]
# The password is generated on the first deploy and kept in deploy/.credentials (gitignored).
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
PROJECT=${1:-$(gcloud config get-value project 2>/dev/null)}
REGION=${2:-us-west1}
SERVICE=chase-settlement-recon
cd "$ROOT"

"$ROOT/scripts/build.sh"

CREDS="$ROOT/deploy/.credentials"
if [ ! -f "$CREDS" ]; then
  printf 'user=juspay\npassword=%s\n' "$(openssl rand -base64 18 | tr -d '/+=' | cut -c1-20)" > "$CREDS"
fi
USER_NAME=$(sed -n 's/^user=//p' "$CREDS"); PASSWORD=$(sed -n 's/^password=//p' "$CREDS")

rm -rf deploy/site && cp -R dist deploy/site
printf '%s:%s\n' "$USER_NAME" "$(openssl passwd -apr1 "$PASSWORD")" > deploy/site/.htpasswd

gcloud run deploy "$SERVICE" --project "$PROJECT" --region "$REGION" --source deploy \
  --port 8080 --memory 256Mi --cpu 1 --min-instances 0 --max-instances 2 \
  --allow-unauthenticated --quiet
echo "Sign in as: $USER_NAME / (password in deploy/.credentials)"
gcloud run services describe "$SERVICE" --project "$PROJECT" --region "$REGION" --format 'value(status.url)'
