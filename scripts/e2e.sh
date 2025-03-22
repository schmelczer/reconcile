#!/bin/bash

set -e
set -o pipefail

# Check if the argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <number_of_processes>"
    exit 1
fi

# Get the number of processes from the first argument
process_count=$1

mkdir -p logs

cd frontend
npm run build

../scripts/utils/wait-for-server.sh

pids=()
for i in $(seq 1 $process_count); do
    node test-client/dist/cli.js > "../logs/log_${i}.log" 2>&1 &
    pids+=($!)
done

cd -

print_failed_log() {
    for i in $(seq 1 $process_count); do
        if [ -n "${pids[$i-1]}" ] && ! kill -0 ${pids[$i-1]} 2>/dev/null; then
            # Get the exit code of the process
            wait ${pids[$i-1]}
            exit_code=$?
            
            # Only consider non-zero exit codes as failures
            if [ $exit_code -ne 0 ]; then
                cat "$(pwd)/logs/log_${i}.log"
                echo "Process ${pids[$i-1]} failed with exit code $exit_code. Log file: $(pwd)/logs/log_${i}.log"
                return 0
            else
                echo "Process ${pids[$i-1]} completed successfully with exit code 0"
                # Mark this PID as processed by setting it to empty
                pids[$i-1]=""
            fi
        fi
    done
    return 1
}

echo "Monitoring $process_count processes"

# Monitor processes
while true; do
    if print_failed_log; then
        # Kill remaining processes
        for pid in "${pids[@]}"; do
            if [ -n "$pid" ]; then
                kill $pid 2>/dev/null || true
            fi
        done
        exit 1
    fi

    # Check if all processes have completed
    all_done=true
    for pid in "${pids[@]}"; do
        if [ -n "$pid" ] && kill -0 $pid 2>/dev/null; then
            all_done=false
            break
        fi
    done

    if $all_done; then
        echo "All processes completed successfully"
        exit 0
    fi

    sleep 0.2
done

