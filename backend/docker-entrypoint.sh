#!/bin/bash

# Disable exit on error for the entire script
set +e

echo "Starting backend container with watch mode..."
echo "Current directory: $(pwd)"
echo "Listing files in current directory:"
ls -la

# Check if cargo-watch is installed
if command -v cargo-watch &> /dev/null; then
    echo "cargo-watch is available, using watch mode for development..."
    
    # Remove Cargo.lock file to avoid version compatibility issues
    echo "Removing Cargo.lock file to avoid version compatibility issues..."
    rm -f Cargo.lock
    echo "Cargo.lock file removed."
    
    # Use cargo-watch to run the application with automatic rebuilding
    echo "Starting cargo-watch in development mode..."
    echo "Watching for changes in src/, migration/, Cargo.toml files..."
    
    # Run cargo-watch with the following options:
    # -w src/ : Watch the src directory
    # -w migration/ : Watch the migration directory  
    # -w Cargo.toml : Watch Cargo.toml for dependency changes
    # -w migration/Cargo.toml : Watch migration Cargo.toml
    # --exec "run" : Execute cargo run when changes are detected
    # --delay 0.5 : Wait 0.5 seconds after changes before rebuilding
    # --clear : Clear the screen before each run
    # --why : Show why cargo-watch is rebuilding
    cargo-watch \
        -w src/ \
        -w migration/ \
        -w Cargo.toml \
        -w migration/Cargo.toml \
        --exec "run" \
        --delay 0.5 \
        --clear \
        --why
else
    echo "cargo-watch not available, falling back to regular cargo run..."
    
    # Remove Cargo.lock file to avoid version compatibility issues
    echo "Removing Cargo.lock file to avoid version compatibility issues..."
    rm -f Cargo.lock
    echo "Cargo.lock file removed."
    
    # Run cargo build first to generate a new Cargo.lock file
    echo "Running: cargo build"
    cargo build
    BUILD_EXIT_CODE=$?
    echo "cargo build exited with code $BUILD_EXIT_CODE"
    
    # Run the application if build was successful
    if [ $BUILD_EXIT_CODE -eq 0 ]; then
        echo "Build successful, running the application..."
        echo "Running: cargo run"
        cargo run
        RUN_EXIT_CODE=$?
        echo "cargo run exited with code $RUN_EXIT_CODE"
    else
        echo "Build failed, not running the application."
    fi
fi

# Keep the container running if all commands fail
echo "All commands have exited. Keeping container alive for debugging..."
tail -f /dev/null