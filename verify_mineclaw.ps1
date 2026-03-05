# verify_mineclaw.ps1
$ErrorActionPreference = "Stop"

# 配置
$MineClawPath = "d:\mineclaw\target\release\mineclaw.exe"
$Port = 18789
$HealthUrl = "http://127.0.0.1:18789/health"

Write-Host ">>> 1. Checking mineclaw.exe..." -ForegroundColor Cyan
if (-not (Test-Path $MineClawPath)) {
    Write-Error "mineclaw.exe not found, please run cargo build --release first"
}

Write-Host ">>> 2. Starting MineClaw Service (Background)..." -ForegroundColor Cyan
$Process = Start-Process -FilePath $MineClawPath -PassThru -NoNewWindow

$ProcessId = $Process.Id
Write-Host "MineClaw Started, PID: $ProcessId" -ForegroundColor Green

try {
    Write-Host ">>> 3. Waiting for service (3s)..." -ForegroundColor Cyan
    Start-Sleep -Seconds 3

    if ($Process.HasExited) {
        Write-Error "MineClaw failed to start. ExitCode: $($Process.ExitCode)"
    } else {
        Write-Host "Service is running (Process active)." -ForegroundColor Green
    }

    Write-Host ">>> 4. Checking Port Listening (netstat)..." -ForegroundColor Cyan
    $NetstatOut = netstat -ano | Select-String ":18789" | Select-String "LISTENING"
    if ($NetstatOut) {
        Write-Host "Port 18789 is LISTENING: $NetstatOut" -ForegroundColor Green
    } else {
        Write-Warning "Port 18789 NOT DETECTED. Service might be slow or blocked."
    }

    Write-Host ">>> 5. Sending HTTP Health Check..." -ForegroundColor Cyan
    try {
        $Response = Invoke-WebRequest -Uri $HealthUrl -UseBasicParsing -TimeoutSec 5
        if ($Response.StatusCode -eq 200) {
            Write-Host "HTTP Request SUCCESS! Status: 200 OK" -ForegroundColor Green
            Write-Host "Response Content: $($Response.Content)" -ForegroundColor Green
        } else {
            Write-Error "HTTP Request FAILED, Status: $($Response.StatusCode)"
        }
    } catch {
        Write-Error "Unable to connect to server: $_"
    }

} finally {
    Write-Host ">>> 6. Cleanup (Stopping MineClaw)..." -ForegroundColor Cyan
    if (-not $Process.HasExited) {
        Stop-Process -Id $ProcessId -Force
        Write-Host "MineClaw Process Terminated" -ForegroundColor Yellow
    }
}
