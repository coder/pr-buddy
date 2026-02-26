# Windows EV code signing script for Tauri signCommand
# Uses jsign with GCP Cloud KMS for EV certificate signing
#
# Called by Tauri as: powershell -File scripts/sign-windows.ps1 <file_path>
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

# Check if signing is configured
if (-not $env:JSIGN_PATH -or -not $env:EV_KEYSTORE) {
    Write-Host "Windows code signing not configured - skipping $FilePath"
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
    if (-not [System.Environment]::GetEnvironmentVariable($varName)) {
        Write-Error "Missing required environment variable: $varName"
        exit 1
    }
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

& java @jsignArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to sign $FilePath (exit code: $LASTEXITCODE)"
    exit 1
}

Write-Host "Successfully signed $FilePath"
