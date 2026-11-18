$ErrorActionPreference = 'Stop'
$arcadeProcesses = @()
$launcherExit = 0
$workspacePath = $PSScriptRoot
$frontendPath = Join-Path $workspacePath 'desktop'

function Wait-ArcadeEndpoint($Url, $Process) {
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $deadline) {
        if ($Process.HasExited) { throw "A server exited during startup. Check the arcade-*.log files." }
        try {
            $response = Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 2
            if ($response.StatusCode -eq 200) { return }
        } catch { }
        Start-Sleep -Milliseconds 200
    }
    throw "Timed out waiting for $Url. Check the arcade-*.log files."
}

Push-Location $workspacePath
try {
    Get-Command cargo.exe, npm.cmd -ErrorAction Stop | Out-Null
    $occupiedPorts = Get-NetTCPConnection -LocalPort 4567,5173 -State Listen -ErrorAction SilentlyContinue
    if ($occupiedPorts) { throw 'Ports 4567 and 5173 must be free before launching. Stop any previous arcade servers first.' }

    Write-Host 'Building the Rust backend...'
    & cargo.exe build -p arcade-server --target-dir target
    if ($LASTEXITCODE -ne 0) { throw 'The Rust build failed.' }

    if (-not (Test-Path -LiteralPath (Join-Path $frontendPath 'node_modules/.bin/vite.cmd'))) {
        Push-Location $frontendPath
        try {
            & npm.cmd ci
            if ($LASTEXITCODE -ne 0) { throw 'Frontend dependency installation failed.' }
        } finally { Pop-Location }
    }

    $backendProcess = Start-Process -FilePath (Join-Path $workspacePath 'target/debug/arcade-server.exe') `
        -WorkingDirectory $workspacePath -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput (Join-Path $workspacePath 'arcade-backend.log') `
        -RedirectStandardError (Join-Path $workspacePath 'arcade-backend-error.log')
    $arcadeProcesses += $backendProcess
    Wait-ArcadeEndpoint 'http://127.0.0.1:4567/api/status' $backendProcess

    $frontendProcess = Start-Process -FilePath $env:ComSpec `
        -ArgumentList '/d', '/c', 'npm.cmd run dev -- --host 127.0.0.1 --port 5173' `
        -WorkingDirectory $frontendPath -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput (Join-Path $workspacePath 'arcade-frontend.log') `
        -RedirectStandardError (Join-Path $workspacePath 'arcade-frontend-error.log')
    $arcadeProcesses += $frontendProcess
    Wait-ArcadeEndpoint 'http://127.0.0.1:5173' $frontendProcess

    Start-Process 'http://127.0.0.1:5173'
    Read-Host 'Arcade is ready. Press Enter here to stop both servers' | Out-Null
} catch {
    Write-Host $_.Exception.Message -ForegroundColor Red
    $launcherExit = 1
} finally {
    foreach ($arcadeProcess in $arcadeProcesses) {
        if (-not $arcadeProcess.HasExited) {
            & taskkill.exe /PID $arcadeProcess.Id /T /F 2>$null | Out-Null
        }
    }
    Pop-Location
}
exit $launcherExit
