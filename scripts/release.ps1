param(
    [Parameter(Mandatory=$false, Position=0)]
    [string]$Version = "minor",

    [Parameter(Mandatory=$false)]
    [switch]$NoPush = $false
)

$ErrorActionPreference = "Stop"
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Write-Utf8NoBom([string]$RelativePath, [string]$Text) {
    $fullPath = Join-Path (Get-Location).Path $RelativePath
    [System.IO.File]::WriteAllText($fullPath, $Text, $utf8NoBom)
}

# 1. Read current version from package.json
$pkgJsonPath = "package.json"
$pkgContent = Get-Content -Raw -Path $pkgJsonPath | ConvertFrom-Json
$currentVersion = $pkgContent.version.TrimStart("v")
$parts = $currentVersion.Split(".") | ForEach-Object { [int]$_ }
if ($parts.Count -lt 3) {
    Write-Error "Invalid current version in package.json: $currentVersion"
    exit 1
}

# 2. Compute target version (supports 'patch', 'minor', 'major', or explicit 'X.Y.Z' / 'vX.Y.Z')
switch ($Version.ToLower()) {
    "patch" {
        $rawVersion = "$($parts[0]).$($parts[1]).$($parts[2] + 1)"
    }
    "minor" {
        $rawVersion = "$($parts[0]).$($parts[1] + 1).0"
    }
    "major" {
        $rawVersion = "$($parts[0] + 1).0.0"
    }
    "" {
        $rawVersion = "$($parts[0]).$($parts[1] + 1).0"
    }
    default {
        $rawVersion = $Version.TrimStart("v")
        if ($rawVersion -notmatch '^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$') {
            Write-Error "Invalid version argument '$Version'. Use 'patch', 'minor', 'major', or 'X.Y.Z'."
            exit 1
        }
    }
}

$tag = "v$rawVersion"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "   OpenBoom Auto-Release & Publish ($currentVersion -> $tag)" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 3. Ensure working tree is clean before bumping
$status = git status --porcelain
if ($status) {
    Write-Warning "Working tree has uncommitted modifications:"
    git status -s
    Write-Error "Please commit or stash changes before running auto-release."
    exit 1
}

# 4. Ensure tag does not already exist
$existingTag = git tag -l $tag
if ($existingTag) {
    Write-Error "Git tag '$tag' already exists locally!"
    exit 1
}

if ($rawVersion -ne $currentVersion) {
    Write-Host "[1/6] Bumping version to $rawVersion across workspace manifests (UTF-8 no BOM)..." -ForegroundColor Green

    # Update package.json
    $pkgRaw = Get-Content -Raw -Path "package.json"
    $pkgRaw = [regex]::Replace($pkgRaw, '"version":\s*"[^"]+"', "`"version`": `"$rawVersion`"", 1)
    Write-Utf8NoBom "package.json" $pkgRaw

    # Update src-ui/package.json
    $uiPkgRaw = Get-Content -Raw -Path "src-ui/package.json"
    $uiPkgRaw = [regex]::Replace($uiPkgRaw, '"version":\s*"[^"]+"', "`"version`": `"$rawVersion`"", 1)
    Write-Utf8NoBom "src-ui/package.json" $uiPkgRaw

    # Update src-tauri/tauri.conf.json
    $tauriConfRaw = Get-Content -Raw -Path "src-tauri/tauri.conf.json"
    $tauriConfRaw = [regex]::Replace($tauriConfRaw, '"version":\s*"[^"]+"', "`"version`": `"$rawVersion`"", 1)
    Write-Utf8NoBom "src-tauri/tauri.conf.json" $tauriConfRaw

    # Update root Cargo.toml ([workspace.package] version)
    $cargoTomlRaw = Get-Content -Raw -Path "Cargo.toml"
    $cargoTomlRaw = [regex]::Replace($cargoTomlRaw, '(?m)^version\s*=\s*"[^"]+"', "version = `"$rawVersion`"", 1)
    Write-Utf8NoBom "Cargo.toml" $cargoTomlRaw

    # Update fallback version string in src-ui/src/components/SettingsModal.tsx
    $settingsModalPath = "src-ui/src/components/SettingsModal.tsx"
    if (Test-Path $settingsModalPath) {
        $settingsModalRaw = Get-Content -Raw -Path $settingsModalPath
        $settingsModalRaw = $settingsModalRaw -replace 'updateInfo\?\.currentVersion \|\| "[^"]+"', "updateInfo?.currentVersion || `"$rawVersion`""
        Write-Utf8NoBom $settingsModalPath $settingsModalRaw
    }

    Write-Host "[2/6] Synchronizing Cargo.lock with workspace version $rawVersion..." -ForegroundColor Green
    cargo update -w
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to update Cargo.lock"
        exit 1
    }

    Write-Host "[3/6] Verifying workspace compilation (cargo check --workspace --locked)..." -ForegroundColor Green
    cargo check --workspace --locked
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Workspace check failed after version bump"
        exit 1
    }

    Write-Host "[4/6] Committing version bump (chore(release): bump version to $tag)..." -ForegroundColor Green
    git add package.json src-ui/package.json src-tauri/tauri.conf.json Cargo.toml Cargo.lock src-ui/src/components/SettingsModal.tsx
    git commit -m "chore(release): bump version to $tag"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to commit version bump"
        exit 1
    }
} else {
    Write-Host "[1/6] Version is already $rawVersion, skipping manifest edits..." -ForegroundColor Yellow
}

Write-Host "[5/6] Creating annotated Git tag: $tag..." -ForegroundColor Green
git tag -a $tag -m "OpenBoom Release $tag"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create tag $tag"
    exit 1
}

if (-not $NoPush) {
    Write-Host "[6/6] Pushing main branch and tag $tag to origin..." -ForegroundColor Green
    git push origin main
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to push main branch to origin"
        exit 1
    }
    git push origin $tag
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to push tag $tag to origin"
        exit 1
    }

    Write-Host ""
    Write-Host "Release $tag published successfully! GitHub Actions (.github/workflows/release.yml) is now building the release assets." -ForegroundColor Cyan
} else {
    Write-Host "[6/6] Skipping git push (-NoPush specified)." -ForegroundColor Yellow
}
