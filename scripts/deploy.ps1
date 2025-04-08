param (
    [switch]$all,
    [switch]$frontend,
    [switch]$backend,
    [switch]$deployOnly,
    [switch]$frontendOnlyDeploy,
    [switch]$backendOnlyDeploy,
    [switch]$norestart
)

# === Setup paths ===
$PROJECT_ROOT = $PSScriptRoot + "\..\" 
$FRONTEND_DIR = "$PROJECT_ROOT\shorten-link-frontend"
$PUBLIC_WWW = "$PROJECT_ROOT\public\www"
$BINARY_NAME = "url-shortener"

# === Load .env ===
$envFile = Join-Path -Path $PSScriptRoot -ChildPath ".env"
if (Test-Path $envFile) {
    Write-Host "Loading environment variables from .env..."
    Get-Content $envFile | ForEach-Object {
        if ($_ -match "^\s*([^#][^=]*)\s*=\s*(.*)\s*$") {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($key, $value)
        }
    }
} else {
    Write-Warning ".env file not found in script directory."
}

# === Get env vars ===
$REMOTE_USER = $env:REMOTE_USER
$REMOTE_HOST = $env:REMOTE_HOST
$REMOTE_PATH = $env:REMOTE_PATH
$SSH_PORT = $env:SSH_PORT
$PROJECT_ROOT_IN_WSL = $env:PROJECT_ROOT_IN_WSL

# === Validate env ===
if (-not $REMOTE_USER -or -not $REMOTE_HOST -or -not $REMOTE_PATH -or -not $SSH_PORT -or -not $PROJECT_ROOT_IN_WSL) {
    Write-Error "Missing one or more required environment variables: REMOTE_USER, REMOTE_HOST, REMOTE_PATH, SSH_PORT, PROJECT_ROOT_IN_WSL"
    exit 1
}

function Build-Frontend {
    Write-Host "`n==> Building frontend..."
    cd "$FRONTEND_DIR"
    pnpm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Frontend build failed."
        exit 1
    }
    cd $PROJECT_ROOT

    Write-Host "==> Copying frontend dist to public/www..."
    Remove-Item -Recurse -Force "$PUBLIC_WWW" -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path "$PUBLIC_WWW" | Out-Null
    Copy-Item -Recurse "$FRONTEND_DIR\dist\*" "$PUBLIC_WWW"
}

function Build-Backend {
    Write-Host "`n==> Building Rust backend..."
    $BUILD_RESULT = wsl --cd "$PROJECT_ROOT_IN_WSL" -- bash -c "source ~/.cargo/env && cargo build --release"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Rust build failed."
        exit 1
    }
}

function Deploy-Backend {
    Write-Host "`n==> Deploying backend to remote server..."
    wsl -- rsync -avz --progress `
        "$PROJECT_ROOT_IN_WSL/target/release/$BINARY_NAME" `
        -e "ssh -p $SSH_PORT" `
        "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}"

    if (-not $norestart) {
        Write-Host "==> Restarting service 'url-shortener' on remote server..."
        wsl -- ssh -p $SSH_PORT "${REMOTE_USER}@${REMOTE_HOST}" "sudo systemctl restart url-shortener"
    } else {
        Write-Host "Skipping service restart (--norestart set)"
    }
}

function Deploy-Frontend {
    Write-Host "`n==> Deploying frontend to remote server..."
    wsl -- rsync -avz --progress `
        "$PROJECT_ROOT_IN_WSL/public/www/" `
        -e "ssh -p $SSH_PORT" `
        "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PATH}/public/www"
}

# === Run selected actions ===
if ($all) {
    Build-Frontend
    Build-Backend
    Deploy-Backend
    Deploy-Frontend
}
elseif ($frontend) {
    Build-Frontend
}
elseif ($backend) {
    Build-Backend
}
elseif ($deployOnly) {
    Deploy-Backend
    Deploy-Frontend
}
elseif ($frontendOnlyDeploy) {
    Deploy-Frontend
}
elseif ($backendOnlyDeploy) {
    Deploy-Backend
}
else {
    Write-Host "`nUsage:"
    Write-Host "  .\deploy.ps1 -all                # build + deploy"
    Write-Host "  .\deploy.ps1 -frontend           # build frontend only"
    Write-Host "  .\deploy.ps1 -backend            # build backend only"
    Write-Host "  .\deploy.ps1 -deploy-only        # only deploy (frontend + backend)"
    Write-Host "  .\deploy.ps1 -frontend-only-deploy # only deploy frontend"
    Write-Host "  .\deploy.ps1 -backend-only-deploy  # only deploy backend"
    Write-HOST "  .\deploy.ps1 -norestart          # do not perform restart of service on the remote server, only works with deploying backend"
    exit 1
}

Write-Host "`n✅ Done!"
