# Windows EV code signing script for Tauri signCommand
# Uses jsign with GCP Cloud KMS for EV certificate signing
#
# Called by Tauri as: pwsh -File scripts/sign-windows.ps1 <file_path>
#
# NOTE: Tauri's bundler swallows stdout/stderr from sign commands on failure
# (output_ok() discards output on non-zero exit). All diagnostic output is
# tee'd to a log file so a post-failure workflow step can dump it.
#
# Required environment variables:
#   JSIGN_PATH          - Path to jsign JAR file
#   EV_KEYSTORE         - GCP Cloud KMS keystore URL
#   EV_KEY              - Key alias in the keystore
#   EV_CERTIFICATE_PATH - Path to the EV certificate PEM file
#   EV_TSA_URL          - Timestamp server URL
#   GCLOUD_ACCESS_TOKEN - GCP access token for authentication

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$FilePath
)

# Log to file since Tauri bundler discards subprocess output on failure
$logDir = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$logFile = Join-Path $logDir "sign-windows.log"
Start-Transcript -Path $logFile -Append -Force | Out-Null

Write-Host "=== sign-windows.ps1 started at $(Get-Date -Format o) ==="
Write-Host "FilePath: $FilePath"
Write-Host "PID: $PID"
Write-Host "PowerShell: $($PSVersionTable.PSVersion)"

# Check if signing is configured
if (-not $env:JSIGN_PATH -or -not $env:EV_KEYSTORE) {
    Write-Host "Windows code signing not configured - skipping $FilePath"
    Stop-Transcript | Out-Null
    exit 0
}

# Validate required environment variables
$requiredVars = @(
    "JSIGN_PATH",
    "EV_KEYSTORE",
    "EV_KEY",
    "EV_CERTIFICATE_PATH",
    "EV_TSA_URL",
    "GCLOUD_ACCESS_TOKEN"
)

foreach ($varName in $requiredVars) {
    $val = [System.Environment]::GetEnvironmentVariable($varName)
    if (-not $val) {
        Write-Error "Missing required environment variable: $varName"
        Stop-Transcript | Out-Null
        exit 1
    }
    # Log presence without leaking secrets
    $display = if ($varName -eq "GCLOUD_ACCESS_TOKEN") { "***($($val.Length) chars)" } else { $val }
    Write-Host "${varName}: $display"
}

# Verify file exists
if (-not (Test-Path $FilePath)) {
    Write-Error "File not found: $FilePath"
    Stop-Transcript | Out-Null
    exit 1
}
Write-Host "File exists: $FilePath ($($(Get-Item $FilePath).Length) bytes)"

# Verify java is available
$javaCmd = Get-Command java -ErrorAction SilentlyContinue
if (-not $javaCmd) {
    Write-Error "java not found in PATH"
    Write-Host "PATH: $env:PATH"
    Stop-Transcript | Out-Null
    exit 1
}
Write-Host "Java: $($javaCmd.Source)"

# Verify jsign jar exists
if (-not (Test-Path $env:JSIGN_PATH)) {
    Write-Error "jsign jar not found: $env:JSIGN_PATH"
    Stop-Transcript | Out-Null
    exit 1
}

# Verify certificate exists
if (-not (Test-Path $env:EV_CERTIFICATE_PATH)) {
    Write-Error "Certificate file not found: $env:EV_CERTIFICATE_PATH"
    Stop-Transcript | Out-Null
    exit 1
}

Write-Host "Signing $FilePath with EV certificate..."

$jsignArgs = @(
    "-jar", $env:JSIGN_PATH,
    "--storetype", "GOOGLECLOUD",
    "--storepass", $env:GCLOUD_ACCESS_TOKEN,
    "--keystore", $env:EV_KEYSTORE,
    "--alias", $env:EV_KEY,
    "--certfile", $env:EV_CERTIFICATE_PATH,
    "--tsmode", "RFC3161",
    "--tsaurl", $env:EV_TSA_URL,
    $FilePath
)

# Log the command (mask the access token)
$displayArgs = $jsignArgs.Clone()
$storepassIdx = [Array]::IndexOf($displayArgs, "--storepass")
if ($storepassIdx -ge 0 -and ($storepassIdx + 1) -lt $displayArgs.Length) {
    $displayArgs[$storepassIdx + 1] = "***"
}
Write-Host "Running: java $($displayArgs -join ' ')"

& java @jsignArgs 2>&1 | ForEach-Object { Write-Host $_ }
$exitCode = $LASTEXITCODE

if ($exitCode -ne 0) {
    Write-Error "Failed to sign $FilePath (exit code: $exitCode)"
    Stop-Transcript | Out-Null
    exit $exitCode
}

Write-Host "Successfully signed $FilePath"
Stop-Transcript | Out-Null
