$PROJECT_ROOT = $PSScriptRoot
$PUBLIC_WWW = "$PROJECT_ROOT\public\www"
$FRONTEND_DIR = "$PROJECT_ROOT\shorten-link-frontend"

# Step 1: Build frontend using pnpm
Write-Host "Building frontend..."
cd "$FRONTEND_DIR"  
pnpm run build
if ($LASTEXITCODE -ne 0) { exit 1 }
cd $PROJECT_ROOT

# Step 2: Copy frontend dist to public/www
Write-Host "Copying frontend dist to public/www..."
Remove-Item -Recurse -Force "$PUBLIC_WWW" 2>$null
New-Item -ItemType Directory -Path "$PUBLIC_WWW" | Out-Null
Copy-Item -Recurse "$FRONTEND_DIR\dist\*" "$PUBLIC_WWW"