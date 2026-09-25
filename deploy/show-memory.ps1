<#
Show what the coach has learned about her, or reset it.

Usage (PowerShell, from the repo folder):
  .\deploy\show-memory.ps1 -Tablet 192.168.199.110           # current memory + recent history
  .\deploy\show-memory.ps1 -Tablet 192.168.199.110 -Reset    # forget everything (history is kept)

To correct the memory by hand, edit the file on the tablet:
  ssh root@192.168.199.110 vi /home/root/ghostwriter/memory.txt
or copy it here, edit it in Notepad, and copy it back:
  scp root@192.168.199.110:/home/root/ghostwriter/memory.txt .
  scp memory.txt root@192.168.199.110:/home/root/ghostwriter/memory.txt
The next tap uses the edited file; no restart needed.
#>
param(
    [string] $Tablet = "10.11.99.1",
    [switch] $Reset
)

$ErrorActionPreference = "Stop"
if ($Reset) {
    ssh "root@$Tablet" "rm -f /home/root/ghostwriter/memory.txt && echo 'Memory reset. The history stays in memory-log.txt.'"
} else {
    ssh "root@$Tablet" "echo '=== memory.txt ==='; cat /home/root/ghostwriter/memory.txt 2>/dev/null || echo '(nothing learned yet)'; echo; echo '=== last entries of memory-log.txt ==='; tail -n 40 /home/root/ghostwriter/memory-log.txt 2>/dev/null || echo '(no history yet)'"
}
