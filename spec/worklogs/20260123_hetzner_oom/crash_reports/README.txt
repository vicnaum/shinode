CRASH REPORT SUMMARY
====================

Date: January 22-23, 2025
Issue: Out of Memory (OOM) Kill
Process: stateless-histo (stateless-history-node)

CRASH EVENTS:
-------------
1. First crash: Jan 22, 23:34:13 UTC
   - PID: 3390463
   - Memory consumed: ~61GB resident (anon-rss: 62554768kB)
   - Virtual memory: ~91GB (total-vm: 93785464kB)
   - Runtime: ~34 minutes

2. Second crash: Jan 23, 00:24:37 UTC
   - PID: 3477330
   - Memory consumed: ~60GB resident (anon-rss: 62222856kB)
   - Virtual memory: ~91GB (total-vm: 93248188kB)
   - Runtime: ~30 minutes

SYSTEM RESOURCES:
-----------------
- Total RAM: 62GB
- Available RAM: ~58GB
- Swap: 31GB (2.9GB used at time of report)
- System was running out of physical memory

FILES IN THIS DIRECTORY:
------------------------
- dmesg_oom.log: Kernel OOM kill messages from dmesg
- journalctl_oom.log: Systemd journal entries related to OOM kills
- crash_window_logs.log: Logs from the crash time window
- memory_info.txt: Current memory status and /proc/meminfo details
- disk_usage.txt: Disk space usage
- swap_info.txt: Swap configuration and usage
- vm_overcommit.txt: VM overcommit settings
- system_info.txt: System version, kernel, CPU info
- top_memory_processes.txt: Current top memory-consuming processes
- trace_files_list.txt: List of trace files generated
- dmesg_recent.log: Recent kernel messages (last 100 lines)

ROOT CAUSE:
-----------
The application was killed by the Linux OOM killer after consuming
~60GB of RAM.
Memory usage grew progressively over ~30 minutes until exceeding
available system RAM.
