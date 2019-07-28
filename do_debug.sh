#!/bin/bash
tmux split-window -h -d 'nc localhost 2333'
gdb-multiarch -q -x jlink.gdb $@
echo "Killing $netcat_pid"
tmux kill-pane -t {right}
