#!/bin/bash

# Spin up the JLink server if its not already running
if ! pgrep -x "JLinkGDBServer" > /dev/null
then
    echo "GDB server not running."
    tmux new-window -d -t 99 "/opt/SEGGER/JLink/JLinkGDBServer -if SWD -speed 4000 -device NRF52840_XXAA"
fi

# Start up Netcat to connect to semihosting output
tmux split-window -h -d 'nc localhost 2333'

# Start GDB debug session.
gdb-multiarch -q -x jlink.gdb $@

# Clean up the created panes.
echo "Killing $netcat_pid"
tmux kill-pane -t {right}
