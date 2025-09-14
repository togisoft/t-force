#!/bin/bash

# Production entrypoint script for tforce backend
set -e

echo "Starting tforce backend in production mode..."
echo "Current directory: $(pwd)"
echo "Environment: ${NODE_ENV:-production}"

# Check if the binary exists
if [ -f "./tforce" ]; then
    echo "Found tforce binary, executing..."
    echo "Binary info:"
    ls -la ./tforce
    file ./tforce
    
    # Run the application
    exec ./tforce
else
    echo "Error: tforce binary not found!"
    echo "Available files:"
    ls -la
    exit 1
fi