Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-UnderWorkspace {
    param(
        [Parameter(Mandatory = $true)][string]$Workspace,
        [Parameter(Mandatory = $true)][string]$Candidate
    )

    $workspacePath = [System.IO.Path]::GetFullPath($Workspace)
    $candidatePath = [System.IO.Path]::GetFullPath($Candidate)
    $workspacePrefix = if ($workspacePath.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $workspacePath
    } else {
        $workspacePath + [System.IO.Path]::DirectorySeparatorChar
    }

    if ($candidatePath -ne $workspacePath -and -not $candidatePath.StartsWith($workspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "refusing to touch path outside workspace: $candidatePath"
    }

    return $candidatePath
}

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

$wildcardPatterns = @(
    "_tmp_*",
    "_recovery*",
    "recovery_*"
)

$literalPaths = @(
    "_probe.c",
    "tests\\reference\\.build"
)

$removed = New-Object System.Collections.Generic.List[string]

foreach ($pattern in $wildcardPatterns) {
    $matches = Get-ChildItem -Path (Join-Path $workspace $pattern) -Force -ErrorAction SilentlyContinue
    foreach ($match in $matches) {
        $target = Assert-UnderWorkspace -Workspace $workspace -Candidate $match.FullName
        Remove-Item -LiteralPath $target -Force -Recurse
        $removed.Add($target) | Out-Null
    }
}

foreach ($path in $literalPaths) {
    $target = Join-Path $workspace $path
    if (Test-Path -LiteralPath $target) {
        $resolved = Assert-UnderWorkspace -Workspace $workspace -Candidate $target
        Remove-Item -LiteralPath $resolved -Force -Recurse
        $removed.Add($resolved) | Out-Null
    }
}

Write-Output "WORKSPACE SCRATCH CLEAN PASS"
Write-Output "removed: $($removed.Count)"
foreach ($entry in $removed) {
    Write-Output " - $entry"
}
