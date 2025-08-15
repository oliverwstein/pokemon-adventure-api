#!/bin/bash
set -e

echo "🚀 Building and deploying Pokemon Adventure API..."

# Build Docker image
echo "📦 Building Docker image..."
docker build --platform linux/arm64 -f pokemon-adventure-api/Dockerfile -t pokemon-api .

# Extract binary directly using docker run
echo "📤 Extracting binary..."
docker run --platform linux/arm64 --rm -v $(pwd):/output pokemon-api cp /var/runtime/bootstrap /output/bootstrap

# Create deployment package
echo "📋 Creating deployment package..."
zip -r pokemon-adventure-api.zip bootstrap

# Deploy to Lambda
echo "☁️ Deploying to AWS Lambda..."
aws lambda update-function-code --function-name pokemon-adventure-api --zip-file fileb://pokemon-adventure-api.zip

# Cleanup
echo "🧹 Cleaning up..."
rm bootstrap pokemon-adventure-api.zip

echo "✅ Deployment complete!"