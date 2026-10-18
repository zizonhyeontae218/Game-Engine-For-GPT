# Resource supervisor for these two exact QA processes only; no application data changes.
$ErrorActionPreference='Stop'
$qa=Split-Path -Parent $MyInvocation.MyCommand.Path
$low=0
$iteration=0
while ($true) {
    $os=Get-CimInstance Win32_OperatingSystem
    $row=@{utc=(Get-Date).ToUniversalTime().ToString('o');free_kib=$os.FreePhysicalMemory;total_kib=$os.TotalVisibleMemorySize}
    # User steering: inspect progress files only every five minutes.
    # Resource-only sampling remains every30seconds to preserve the existing stop cap.
    if($iteration % 10 -eq 0) {
        foreach($mode in @('headless','linked')) {
            $row[$mode]=Get-Content -LiteralPath "$qa/endurance-$mode.jsonl" -Tail 1 | ConvertFrom-Json
        }
    }
    $row | ConvertTo-Json -Depth 8 -Compress | Add-Content -LiteralPath "$qa/host-monitor.jsonl"
    if($os.FreePhysicalMemory -lt 524288){$low++}else{$low=0}
    if($low -ge 2){
        $targets=Get-CimInstance Win32_Process | Where-Object { $_.Name -eq 'python.exe' -and $_.CommandLine -match 'native_verify.py endurance --mode (headless|linked)' }
        foreach($target in $targets){ & taskkill.exe /PID $target.ProcessId /T /F }
        'STOPPED: host available memory below512MiB for two30-second samples' | Set-Content "$qa/resource-stop.txt"
        exit 2
    }
    if($iteration % 10 -eq 0 -and (Test-Path "$qa/endurance-headless.json") -and (Test-Path "$qa/endurance-linked.json")){exit 0}
    $iteration++
    Start-Sleep -Seconds 30
}
