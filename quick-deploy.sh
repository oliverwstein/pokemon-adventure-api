#!/bin/bash
set -e

echo "⚡ Quick Deploy - Pokemon Adventure API"

# Build and extract in one step
echo "📦 Building and extracting..."
docker build --platform linux/arm64 -f pokemon-adventure-api/Dockerfile.deploy -t pokemon-deploy . && \
docker run --rm -v $(pwd):/output pokemon-deploy sh -c "cp /pokemon-adventure-api.zip /output/"

# Deploy immediately
echo "☁️ Deploying..."
aws lambda update-function-code --function-name pokemon-adventure-api --zip-file fileb://pokemon-adventure-api.zip

# Cleanup
rm pokemon-adventure-api.zip

echo "🎉 Deployed in one command!"