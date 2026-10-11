param(
    [Parameter(Mandatory=$false)]
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

# Determine target version
if ([string]::IsNullOrWhiteSpace($Version)) {
    $pkgJson = Get-Content -Raw -Path "package.json" | ConvertFrom-Json
    $Version = $pkgJson.version
}

# Ensure clean format without 'v' prefix for internal version
$rawVersion = $Version.TrimStart("v")
$tag = "v$rawVersion"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   OpenBoom Release Tag Helper ($tag)    " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Check git status
$status = git status --porcelain
if ($status) {
    Write-Warning "Working tree has uncommitted modifications:"
    git status -s
    Write-Warning "Commit or stash changes before creating a release tag."
    exit 1
}

# Check if tag already exists
$existingTag = git tag -l $tag
if ($existingTag) {
    Write-Error "Git tag '$tag' already exists locally!"
    exit 1
}

Write-Host "[1/3] Creating annotated Git tag: $tag" -ForegroundColor Green
git tag -a $tag -m "OpenBoom Release $tag"

Write-Host "[2/3] Tag $tag created successfully!" -ForegroundColor Green
Write-Host "[3/3] Next steps to trigger automated GitHub Actions build & release:" -ForegroundColor Yellow
Write-Host ""
Write-Host "    git push origin main" -ForegroundColor White
Write-Host "    git push origin $tag" -ForegroundColor Cyan
Write-Host ""
Write-Host "The GitHub Actions workflow (.github/workflows/release.yml) will automatically:" -ForegroundColor Gray
Write-Host "  - Run all Cargo and UI tests" -ForegroundColor Gray
Write-Host "  - Compile OpenBoom in release mode" -ForegroundColor Gray
Write-Host "  - Generate the NSIS installer (OpenBoom_$rawVersion`_x64-setup.exe)" -ForegroundColor Gray
Write-Host "  - Generate the standalone portable executable (OpenBoom-$tag-portable-x64.exe)" -ForegroundColor Gray
Write-Host "  - Compute SHA256 checksums" -ForegroundColor Gray
Write-Host "  - Create GitHub Release and attach the assets" -ForegroundColor Gray
