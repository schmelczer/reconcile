#!/bin/bash

set -e

SERVER_URL="http://localhost:3000"
MAX_RETRIES=30
RETRY_INTERVAL_IN_SECONDS=5

echo "Waiting for $SERVER_URL to become available..."
count=0
while [ $count -lt $MAX_RETRIES ]; do
  if curl -s -f -o /dev/null $SERVER_URL; then
    echo "$SERVER_URL is now available!"
    break
  fi
  echo "Attempt $(($count+1))/$MAX_RETRIES: $SERVER_URL not available yet, retrying in ${RETRY_INTERVAL_IN_SECONDS}s..."
  sleep $RETRY_INTERVAL_IN_SECONDS
  count=$(($count+1))
done

if [ $count -eq $MAX_RETRIES ]; then
  echo "Error: $SERVER_URL did not become available after $MAX_RETRIES attempts."
  exit 1
fi
